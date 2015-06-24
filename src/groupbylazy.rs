use Itertools;
use std::cmp;
use std::cell::{Cell, RefCell};
use std::mem;
use std::vec;

struct GroupInner<K, I, F>
    where I: Iterator,
{
    key: F,
    iter: I,
    current_key: Option<K>,
    current_elt: Option<I::Item>,
    // buffering stuff
    done: bool,
    /// Index of group we are currently buffering or visiting
    top: usize,
    /// Least index for which we still have elements buffered
    bot: usize,

    /// Buffered groups, from `bot` (index 0) to `top`.
    buffer: Vec<vec::IntoIter<I::Item>>,
    /// index of last group iter that was dropped
    dropped_group: Option<usize>,
}

impl<K, I, F> GroupInner<K, I, F>
    where I: Iterator,
          F: FnMut(&I::Item) -> K,
          K: PartialEq,
{
    fn push_next_group(&mut self, group: Vec<I::Item>) {
        // When we add a new buffered group, fill up slots between bot and top
        while self.top - self.bot > self.buffer.len() {
            if self.buffer.is_empty() {
                self.bot += 1;
            } else {
                self.buffer.push(Vec::new().into_iter());
            }
        }
        self.buffer.push(group.into_iter());
        debug_assert!(self.top + 1 - self.bot == self.buffer.len());
    }
    /// `client`: Index of group that requests next element
    fn step(&mut self, client: usize) -> Option<I::Item> {
        /*
        println!("client={}, bot={}, top={}, buffers={:?}",
                 client, self.bot, self.top,
                 self.buffer.iter().map(|x| x.len()).collect::<Vec<_>>());
         */
        if client < self.bot {
            None
        } else if client < self.top ||
            (client == self.top && self.buffer.len() > self.top - self.bot)
        {
            // if `bufidx` doesn't exist in self.buffer, it might be empty
            let bufidx = client - self.bot;
            let elt = self.buffer.get_mut(bufidx).and_then(|queue| queue.next());
            if elt.is_none() {
                // FIXME: Use a smarter way to reuse the vector space
                // VecDeque is unfortunately not zero allocation when empty.
                while self.buffer.len() > 0 && self.buffer[0].len() == 0 {
                    self.buffer.remove(0);
                    self.bot += 1;
                }
            }
            elt
        } else if self.done {
            return None;
        } else if self.top == client {
            self.step_current()
        } else {
            // requested a later group -- walk through all groups up to
            // the requested group index, and buffer the elements (unless
            // the group is marked as dropped).
            let mut group = Vec::new();

            if let Some(elt) = self.current_elt.take() {
                if self.dropped_group != Some(self.top) {
                    group.push(elt);
                }
            }
            loop {
                match self.iter.next() {
                    None => {
                        if self.dropped_group != Some(self.top) {
                            self.push_next_group(group);
                        }
                        self.done = true;
                        return None;
                    }
                    Some(elt) => {
                        let key = (self.key)(&elt);
                        match self.current_key.take() {
                            None => {}
                            Some(old_key) => if old_key != key {
                                if self.dropped_group != Some(self.top) {
                                    self.push_next_group(mem::replace(&mut group, Vec::new()));
                                }
                                self.top += 1;
                                if self.top == client {
                                    self.current_key = Some(key);
                                    return Some(elt);
                                }
                            },
                        }
                        self.current_key = Some(key);
                        if self.dropped_group != Some(self.top) {
                            group.push(elt);
                        }
                    }
                }
            }
        }
    }

    /// This is the immediate case, where we use no buffering
    fn step_current(&mut self) -> Option<I::Item> {
        debug_assert!(!self.done);
        if let elt @ Some(..) = self.current_elt.take() {
            return elt;
        }
        match self.iter.next() {
            None => {
                self.done = true;
                return None;
            }
            Some(elt) => {
                let key = (self.key)(&elt);
                match self.current_key.take() {
                    None => {}
                    Some(old_key) => if old_key != key {
                        self.current_key = Some(key);
                        self.current_elt = Some(elt);
                        self.top += 1;
                        return None;
                    },
                }
                self.current_key = Some(key);
                Some(elt)
            }
        }
    }

    /// Request the just started groups' key.
    ///
    /// `client`: Index of group
    ///
    /// **Panics** if no group key is available.
    fn group_key(&mut self, client: usize) -> K {
        // This can only be called after we have just returned the first
        // element of a group.
        // Perform this by simply buffering one more element, grabbing the
        // next key.
        debug_assert!(!self.done);
        debug_assert!(client == self.top);
        debug_assert!(self.current_key.is_some());
        debug_assert!(self.current_elt.is_none());
        match self.iter.next() {
            None => {
                self.done = true;
                self.current_key.take().unwrap()
            }
            Some(elt) => {
                let key = (self.key)(&elt);
                match self.current_key.take() {
                    None => unreachable!(),
                    Some(old_key) => {
                        if old_key != key {
                            self.top += 1;
                        }
                        self.current_key = Some(key);
                        self.current_elt = Some(elt);
                        old_key
                    }
                }
            }
        }
    }
}

impl<K, I, F> GroupInner<K, I, F>
    where I: Iterator,
{
    /// Called when a group is dropped
    fn drop_group(&mut self, client: usize) {
        // It's only useful to track the highest index
        self.dropped_group = Some(cmp::max(client,
                                           self.dropped_group.unwrap_or(0)));
    }
}

/// `GroupByLazy` is the storage for the lazy grouping operation.
///
/// If the groups are consumed in their original order, or if each
/// group is dropped without keeping it around, then `GroupByLazy` uses
/// no allocations. It needs allocations only if several group iterators
/// are alive at the same time.
///
/// This type implements `IntoIterator` (it is **not** an iterator
/// itself), because the group iterators need to borrow from this
/// value. It should stored in a local variable or temporary and
/// iterated.
///
/// See [`.group_by_lazy()`](trait.Itertools.html#method.group_by_lazy) for more information.
pub struct GroupByLazy<K, I, F>
    where I: Iterator,
{
    inner: RefCell<GroupInner<K, I, F>>,
    // the group iterator's current index. Keep this in the main value
    // so that simultaneous iterators all use the same state.
    index: Cell<usize>,
}

/// Create a new
pub fn new<K, J, F>(iter: J, f: F) -> GroupByLazy<K, J::IntoIter, F>
    where J: IntoIterator,
          F: FnMut(&J::Item) -> K,
{
    GroupByLazy {
        inner: RefCell::new(GroupInner {
            key: f,
            iter: iter.into_iter(),
            current_key: None,
            current_elt: None,
            done: false,
            top: 0,
            bot: 0,
            buffer: Vec::new(),
            dropped_group: None,
        }),
        index: Cell::new(0),
    }
}

impl<K, I, F> GroupByLazy<K, I, F>
    where I: Iterator,
{
    /// `client`: Index of group that requests next element
    fn step(&self, client: usize) -> Option<I::Item>
        where F: FnMut(&I::Item) -> K,
              K: PartialEq,
    {
        self.inner.borrow_mut().step(client)
    }

    /// `client`: Index of group that requests next element
    fn group_key(&self, client: usize) -> K
        where F: FnMut(&I::Item) -> K,
              K: PartialEq,
    {
        self.inner.borrow_mut().group_key(client)
    }

    /// `client`: Index of group
    fn drop_group(&self, client: usize) {
        self.inner.borrow_mut().drop_group(client)
    }
}

impl<'a, K, I, F> IntoIterator for &'a GroupByLazy<K, I, F>
    where I: Iterator,
          I::Item: 'a,
          F: FnMut(&I::Item) -> K,
          K: PartialEq,
{
    type Item = (K, Group<'a, K, I, F>);
    type IntoIter = Groups<'a, K, I, F>;

    fn into_iter(self) -> Self::IntoIter {
        Groups {
            parent: self,
        }
    }
}


/// An iterator that yields the Group iterators.
///
/// Iterator element type is `(K, Group)`:
/// the group's key `K` and the group's iterator.
pub struct Groups<'a, K: 'a, I: 'a, F: 'a>
    where I: Iterator,
          I::Item: 'a,
{
    parent: &'a GroupByLazy<K, I, F>,
}

impl<'a, K, I, F> Iterator for Groups<'a, K, I, F>
    where I: Iterator,
          I::Item: 'a,
          F: FnMut(&I::Item) -> K,
          K: PartialEq,
{
    type Item = (K, Group<'a, K, I, F>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let index = self.parent.index.get();
        self.parent.index.set(index + 1);
        self.parent.step(index).map(|elt| {
            let key = self.parent.group_key(index);
            (key, Group {
                parent: self.parent,
                index: index,
                first: Some(elt),
            })
        })
    }
}

/// An iterator for the elements in a single group.
///
/// Iterator element type is `I::Item`.
pub struct Group<'a, K: 'a, I: 'a, F: 'a>
    where I: Iterator,
          I::Item: 'a,
{
    parent: &'a GroupByLazy<K, I, F>,
    index: usize,
    first: Option<I::Item>,
}

impl<'a, K, I, F> Drop for Group<'a, K, I, F>
    where I: Iterator,
          I::Item: 'a,
{
    fn drop(&mut self) {
        self.parent.drop_group(self.index);
    }
}

impl<'a, K, I, F> Iterator for Group<'a, K, I, F>
    where I: Iterator,
          I::Item: 'a,
          F: FnMut(&I::Item) -> K,
          K: PartialEq,
{
    type Item = I::Item;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let elt @ Some(..) = self.first.take() {
            return elt;
        }
        self.parent.step(self.index)
    }
}
