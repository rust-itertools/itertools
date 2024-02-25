use alloc::vec::{self, Vec};
use std::cell::{Cell, RefCell};

#[derive(Clone)]
struct UnfusedInner<I>
where
    I: Iterator,
{
    iter: I,
    current_index: usize,
    current_elt: Option<I::Item>,
    /// flag set if iterator is exhausted
    done: bool,
    last_was_none: bool,
    /// Index of group we are currently buffering or visiting
    top_group: usize,
    /// Least index for which we still have elements buffered
    oldest_buffered_group: usize,
    /// Group index for `buffer[0]` -- the slots
    /// bottom_group..oldest_buffered_group are unused and will be erased when
    /// that range is large enough.
    bottom_group: usize,
    /// Buffered groups, from `bottom_group` (index 0) to `top_group`.
    buffer: Vec<vec::IntoIter<I::Item>>,
    /// index of last group iter that was dropped, usize::MAX == none
    dropped_group: usize,
}

impl<I> UnfusedInner<I>
where
    I: Iterator,
{
    /// `client`: Index of group that requests next element
    #[inline(always)]
    fn step(&mut self, client: usize) -> Option<I::Item> {
        /*
        println!("client={}, bottom_group={}, oldest_buffered_group={}, top_group={}, buffers=[{}]",
                 client, self.bottom_group, self.oldest_buffered_group,
                 self.top_group,
                 self.buffer.iter().map(|elt| elt.len()).format(", "));
        */
        if client < self.oldest_buffered_group {
            None
        } else if client < self.top_group
            || (client == self.top_group && self.buffer.len() > self.top_group - self.bottom_group)
        {
            self.lookup_buffer(client)
        } else if self.done {
            None
        } else if self.top_group == client {
            self.step_current()
        } else {
            self.step_buffering(client)
        }
    }

    #[inline(never)]
    fn lookup_buffer(&mut self, client: usize) -> Option<I::Item> {
        // if `bufidx` doesn't exist in self.buffer, it might be empty
        let bufidx = client - self.bottom_group;
        if client < self.oldest_buffered_group {
            return None;
        }
        let elt = self.buffer.get_mut(bufidx).and_then(|queue| queue.next());
        if elt.is_none() && client == self.oldest_buffered_group {
            // FIXME: VecDeque is unfortunately not zero allocation when empty,
            // so we do this job manually.
            // `bottom_group..oldest_buffered_group` is unused, and if it's large enough, erase it.
            self.oldest_buffered_group += 1;
            // skip forward further empty queues too
            while self
                .buffer
                .get(self.oldest_buffered_group - self.bottom_group)
                .map_or(false, |buf| buf.len() == 0)
            {
                self.oldest_buffered_group += 1;
            }

            let nclear = self.oldest_buffered_group - self.bottom_group;
            if nclear > 0 && nclear >= self.buffer.len() / 2 {
                let mut i = 0;
                self.buffer.retain(|buf| {
                    i += 1;
                    debug_assert!(buf.len() == 0 || i > nclear);
                    i > nclear
                });
                self.bottom_group = self.oldest_buffered_group;
            }
        }
        elt
    }

    /// Take the next element from the iterator, and set the done
    /// flag if exhausted. Must not be called after done.
    #[inline(always)]
    fn next_element(&mut self) -> Option<I::Item> {
        debug_assert!(!self.done);
        match self.iter.next() {
            None => {
                if self.last_was_none {
                    self.done = true;
                }
                self.last_was_none = true;
                None
            }
            otherwise => {
                self.last_was_none = false;
                otherwise
            }
        }
    }

    #[inline(never)]
    fn step_buffering(&mut self, client: usize) -> Option<I::Item> {
        // requested a later group -- walk through the current group up to
        // the requested group index, and buffer the elements (unless
        // the group is marked as dropped).
        // Because the `Groups` iterator is always the first to request
        // each group index, client is the next index efter top_group.
        debug_assert!(self.top_group + 1 == client);
        let mut group = Vec::new();

        if let Some(elt) = self.current_elt.take() {
            if self.top_group != self.dropped_group {
                group.push(elt);
            }
        }

        loop {
            match self.next_element() {
                Some(elt) => {
                    if self.top_group != self.dropped_group {
                        group.push(elt);
                    }
                },
                None => {
                    self.current_index += 1;
                    break;
                }
            }
        }
        let first_elt = self.next_element();

        if self.top_group != self.dropped_group {
            self.push_next_group(group);
        }
        if first_elt.is_some() {
            self.top_group += 1;
            debug_assert!(self.top_group == client);
        }
        first_elt
    }

    fn push_next_group(&mut self, group: Vec<I::Item>) {
        // When we add a new buffered group, fill up slots between oldest_buffered_group and top_group
        while self.top_group - self.bottom_group > self.buffer.len() {
            if self.buffer.is_empty() {
                self.bottom_group += 1;
                self.oldest_buffered_group += 1;
            } else {
                self.buffer.push(Vec::new().into_iter());
            }
        }
        self.buffer.push(group.into_iter());
        debug_assert!(self.top_group + 1 - self.bottom_group == self.buffer.len());
    }

    /// This is the immediate case, where we use no buffering
    #[inline]
    fn step_current(&mut self) -> Option<I::Item> {
        debug_assert!(!self.done);
        if let elt @ Some(..) = self.current_elt.take() {
            return elt;
        }
        let elt = self.next_element();
        if elt.is_none() {
            self.top_group += 1;
        }
        elt
    }

    // Request the just started groups' key.
    //
    // `client`: Index of group
    //
    // **Panics** if no group key is available.
    // fn group_key(&mut self, client: usize) -> K {
    //     // This can only be called after we have just returned the first
    //     // element of a group.
    //     // Perform this by simply buffering one more element, grabbing the
    //     // next key.
    //     debug_assert!(!self.done);
    //     debug_assert!(client == self.top_group);
    //     debug_assert!(self.current_key.is_some());
    //     debug_assert!(self.current_elt.is_none());
    //     let old_key = self.current_key.take().unwrap();
    //     if let Some(elt) = self.next_element() {
    //         let key = self.key.call_mut(&elt);
    //         if old_key != key {
    //             self.top_group += 1;
    //         }
    //         self.current_key = Some(key);
    //         self.current_elt = Some(elt);
    //     }
    //     old_key
    // }
}

impl<I> UnfusedInner<I>
where
    I: Iterator,
{
    /// Called when a group is dropped
    fn drop_group(&mut self, client: usize) {
        // It's only useful to track the maximal index
        if self.dropped_group == !0 || client > self.dropped_group {
            self.dropped_group = client;
        }
    }
}

/// `SplitUnfused` is the storage for the lazy grouping operation.
///
/// If the groups are consumed in their original order, or if each
/// group is dropped without keeping it around, then `SplitUnfused` uses
/// no allocations. It needs allocations only if several group iterators
/// are alive at the same time.
///
/// This type implements [`IntoIterator`] (it is **not** an iterator
/// itself), because the group iterators need to borrow from this
/// value. It should be stored in a local variable or temporary and
/// iterated.
///
/// See [`.group_by()`](crate::Itertools::group_by) for more information.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct SplitUnfused<I>
where
    I: Iterator,
{
    inner: RefCell<UnfusedInner<I>>,
    // the group iterator's current index. Keep this in the main value
    // so that simultaneous iterators all use the same state.
    index: Cell<usize>,
}

/// Create a new
pub fn new<J>(iter: J) -> SplitUnfused<J::IntoIter>
where
    J: IntoIterator,
{
    SplitUnfused {
        inner: RefCell::new(UnfusedInner {
            // key: f,
            iter: iter.into_iter(),
            current_index: 0,
            current_elt: None,
            done: false,
            last_was_none: false,
            top_group: 0,
            oldest_buffered_group: 0,
            bottom_group: 0,
            buffer: Vec::new(),
            dropped_group: !0,
        }),
        index: Cell::new(0),
    }
}

impl<I> SplitUnfused<I>
where
    I: Iterator,
{
    /// `client`: Index of group that requests next element
    fn step(&self, client: usize) -> Option<I::Item>
    {
        self.inner.borrow_mut().step(client)
    }

    /// `client`: Index of group
    fn drop_group(&self, client: usize) {
        self.inner.borrow_mut().drop_group(client);
    }
}

impl<'a, I> IntoIterator for &'a SplitUnfused<I>
where
    I: Iterator,
    I::Item: 'a,
{
    type Item = Group<'a, I>;
    type IntoIter = Groups<'a, I>;

    fn into_iter(self) -> Self::IntoIter {
        Groups { parent: self }
    }
}

/// An iterator that yields the Group iterators.
///
/// Iterator element type is `(K, Group)`:
/// the group's key `K` and the group's iterator.
///
/// See [`.group_by()`](crate::Itertools::group_by) for more information.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct Groups<'a, I: 'a>
where
    I: Iterator,
    I::Item: 'a,
{
    parent: &'a SplitUnfused<I>,
}

impl<'a, I> Iterator for Groups<'a, I>
where
    I: Iterator,
    I::Item: 'a,
{
    type Item = Group<'a, I>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let index = self.parent.index.get();
        self.parent.index.set(index + 1);
        let inner = &mut *self.parent.inner.borrow_mut();
        inner.step(index).map(|elt| {
            Group {
                parent: self.parent,
                index,
                first: Some(elt),
            }
        })
    }
}

/// An iterator for the elements in a single group.
///
/// Iterator element type is `I::Item`.
pub struct Group<'a, I: 'a>
where
    I: Iterator,
    I::Item: 'a,
{
    parent: &'a SplitUnfused<I>,
    index: usize,
    first: Option<I::Item>,
}

impl<'a, I> Drop for Group<'a, I>
where
    I: Iterator,
    I::Item: 'a,
{
    fn drop(&mut self) {
        self.parent.drop_group(self.index);
    }
}

impl<'a, I> Iterator for Group<'a, I>
where
    I: Iterator,
    I::Item: 'a,
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
