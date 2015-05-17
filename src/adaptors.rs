//! Licensed under the Apache License, Version 2.0
//! http://www.apache.org/licenses/LICENSE-2.0 or the MIT license
//! http://opensource.org/licenses/MIT, at your
//! option. This file may not be copied, modified, or distributed
//! except according to those terms.

use std::mem;
#[cfg(feature = "unstable")]
use std::num::One;
#[cfg(feature = "unstable")]
use std::ops::Add;
use std::cmp::Ordering;
use std::iter::{Fuse, Peekable};
use Itertools;
use size_hint;

macro_rules! clone_fields {
    ($name:ident, $base:expr, $($field:ident),+) => (
        $name {
            $(
                $field : $base . $field .clone()
            ),*
        }
    );
}


/// An iterator adaptor that alternates elements from two iterators until both
/// run out.
///
/// This iterator is *fused*.
///
/// See [*.interleave()*](trait.Itertools.html#method.interleave) for more information.
#[derive(Clone)]
pub struct Interleave<I, J> {
    a: Fuse<I>,
    b: Fuse<J>,
    flag: bool,
}

impl<I, J> Interleave<I, J> where
    I: Iterator,
    J: Iterator,
{
    /// Creat a new **Interleave** iterator.
    pub fn new(a: I, b: J) -> Interleave<I, J> {
        Interleave{a: a.fuse(), b: b.fuse(), flag: false}
    }
}

impl<I, J> Iterator for Interleave<I, J> where
    I: Iterator,
    J: Iterator<Item=I::Item>,
{
    type Item = I::Item;
    #[inline]
    fn next(&mut self) -> Option<I::Item> {
        self.flag = !self.flag;
        if self.flag {
            match self.a.next() {
                None => self.b.next(),
                r => r,
            }
        } else {
            match self.b.next() {
                None => self.a.next(),
                r => r,
            }
        }
    }
}

/// **Deprecated:** Use *.map_fn()* instead.
pub struct FnMap<B, I> where
    I: Iterator,
{
    map: fn(I::Item) -> B,
    iter: I,
}

impl<B, I> FnMap<B, I> where
    I: Iterator
{
    /// **Deprecated:** Use *.map_fn()* instead.
    pub fn new(iter: I, map: fn(I::Item) -> B) -> Self
    {
        FnMap{iter: iter, map: map}
    }
}

impl<B, I> Iterator for FnMap<B, I> where
    I: Iterator,
{
    type Item = B;
    #[inline]
    fn next(&mut self) -> Option<B>
    {
        self.iter.next().map(|a| (self.map)(a))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<B, I> DoubleEndedIterator for FnMap<B, I> where
    I: DoubleEndedIterator
{
    #[inline]
    fn next_back(&mut self) -> Option<B> {
        self.iter.next_back().map(|a| (self.map)(a))
    }
}

// same size
impl<B, I> ExactSizeIterator for FnMap<B, I> where
    I: ExactSizeIterator,
{ }

impl<B, I> Clone for FnMap<B, I> where
    I: Clone + Iterator,
{
    fn clone(&self) -> Self
    {
        FnMap::new(self.iter.clone(), self.map)
    }
}

#[derive(Clone)]
/// An iterator adaptor that allows putting back a single
/// item to the front of the iterator.
///
/// Iterator element type is **I::Item**.
pub struct PutBack<I> where
    I: Iterator,
{
    top: Option<I::Item>,
    iter: I,
}

impl<I> PutBack<I> where
    I: Iterator,
{
    /// Iterator element type is `A`
    #[inline]
    pub fn new(it: I) -> Self
    {
        PutBack{top: None, iter: it}
    }

    /// Put back a single value to the front of the iterator.
    ///
    /// If a value is already in the put back slot, it is overwritten.
    #[inline]
    pub fn put_back(&mut self, x: I::Item)
    {
        self.top = Some(x)
    }
}

impl<I> Iterator for PutBack<I> where
    I: Iterator,
{
    type Item = I::Item;
    #[inline]
    fn next(&mut self) -> Option<I::Item> {
        match self.top {
            None => self.iter.next(),
            ref mut some => some.take(),
        }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // Not ExactSizeIterator because size may be larger than usize
        size_hint::add_scalar(self.iter.size_hint(), self.top.is_some() as usize)
    }
}

/// An iterator adaptor that allows putting multiple
/// items in front of the iterator.
///
/// Iterator element type is **I::Item**.
pub struct PutBackN<I: Iterator>
{
    top: Vec<I::Item>,
    iter: I
}

impl<I: Iterator> PutBackN<I>
{
    /// Iterator element type is `A`
    #[inline]
    pub fn new(it: I) -> Self
    {
        PutBackN{top: vec![], iter: it}
    }

    /// Puts x in front of the iterator.
    /// The values are yielded in order.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use itertools::PutBackN;
    ///
    /// let mut it = PutBackN::new(1..5);
    /// it.next();
    /// it.put_back(1);
    /// it.put_back(0);
    ///
    /// assert!(itertools::equal(it, 0..5));
    /// ```
    #[inline]
    pub fn put_back(&mut self, x: I::Item)
    {
        self.top.push(x);
    }
}

impl<I: Iterator> Iterator for PutBackN<I>
{
    type Item = I::Item;
    #[inline]
    fn next(&mut self) -> Option<I::Item> {
        if self.top.is_empty() {
            self.iter.next()
        } else {
            self.top.pop()
        }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lo, hi) = self.iter.size_hint();
        (lo.saturating_add(self.top.len()), hi.and_then(|x| x.checked_add(self.top.len())))
    }
}

impl<I: Iterator> Clone for PutBackN<I> where
    I: Clone,
    I::Item: Clone
{
    fn clone(&self) -> Self
    {
        clone_fields!(PutBackN, self, top, iter)
    }
}

#[derive(Clone)]
/// An iterator adaptor that iterates over the cartesian product of
/// the element sets of two iterators **I** and **J**.
///
/// Iterator element type is **(I::Item, J::Item)**.
///
/// See [*.cartesian_product()*](trait.Itertools.html#method.cartesian_product) for more information.
pub struct Product<I, J> where
    I: Iterator,
{
    a: I,
    a_cur: Option<I::Item>,
    b: J,
    b_orig: J,
}

impl<I, J> Product<I, J> where
    I: Iterator,
    J: Clone + Iterator,
    I::Item: Clone,
{
    /// Create a new cartesian product iterator
    ///
    /// Iterator element type is **(I::Item, J::Item)**.
    pub fn new(i: I, j: J) -> Self
    {
        let mut i = i;
        Product{a_cur: i.next(), a: i, b: j.clone(), b_orig: j}
    }
}


impl<I, J> Iterator for Product<I, J> where
    I: Iterator,
    J: Clone + Iterator,
    I::Item: Clone,
{
    type Item = (I::Item, J::Item);
    fn next(&mut self) -> Option<(I::Item, J::Item)>
    {
        let elt_b = match self.b.next() {
            None => {
                self.b = self.b_orig.clone();
                match self.b.next() {
                    None => return None,
                    Some(x) => {
                        self.a_cur = self.a.next();
                        x
                    }
                }
            }
            Some(x) => x
        };
        match self.a_cur {
            None => None,
            Some(ref a) => {
                Some((a.clone(), elt_b))
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>)
    {
        let has_cur = self.a_cur.is_some() as usize;
        // Not ExactSizeIterator because size may be larger than usize
        let (b, _) = self.b.size_hint();

        // Compute a * b_orig + b for both lower and upper bound
        size_hint::add_scalar(
            size_hint::mul(self.a.size_hint(), self.b_orig.size_hint()),
            b * has_cur)
    }
}

/// A “meta iterator adaptor”. Its closure recives a reference to the iterator
/// and may pick off as many elements as it likes, to produce the next iterator element.
///
/// Iterator element type is *X*, if the return type of **F** is *Option\<X\>*.
///
/// See [*.batching()*](trait.Itertools.html#method.batching) for more information.
#[derive(Clone)]
pub struct Batching<I, F> {
    f: F,
    iter: I,
}

impl<F, I> Batching<I, F> {
    /// Create a new Batching iterator.
    pub fn new(iter: I, f: F) -> Batching<I, F>
    {
        Batching{f: f, iter: iter}
    }
}

impl<B, F, I> Iterator for Batching<I, F> where
    I: Iterator,
    F: FnMut(&mut I) -> Option<B>,
{
    type Item = B;
    #[inline]
    fn next(&mut self) -> Option<B>
    {
        (self.f)(&mut self.iter)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>)
    {
        // No information about closue behavior
        (0, None)
    }
}

#[derive(Clone)]
/// An iterator adaptor that groups iterator elements. Consecutive elements
/// that map to the same key (“runs”), are returned as the iterator elements.
///
/// See [*.group_by()*](trait.Itertools.html#method.group_by) for more information.
pub struct GroupBy<K, I, F> where
    I: Iterator,
{
    key: F,
    iter: I,
    current_key: Option<K>,
    elts: Vec<I::Item>,
}

impl<K, F, I> GroupBy<K, I, F> where
    I: Iterator,
{
    /// Create a new **GroupBy** iterator.
    pub fn new(iter: I, key: F) -> Self
    {
        GroupBy{key: key, iter: iter, current_key: None, elts: Vec::new()}
    }
}

impl<K, I, F> Iterator for GroupBy<K, I, F> where
    K: PartialEq,
    I: Iterator,
    F: FnMut(&I::Item) -> K,
{
    type Item = (K, Vec<I::Item>);
    fn next(&mut self) -> Option<(K, Vec<I::Item>)>
    {
        for elt in self.iter.by_ref() {
            let key = (self.key)(&elt);
            match self.current_key.take() {
                None => {}
                Some(old_key) => if old_key != key {
                    self.current_key = Some(key);
                    let v = mem::replace(&mut self.elts, vec![elt]);
                    return Some((old_key, v))
                },
            }
            self.current_key = Some(key);
            self.elts.push(elt);
        }
        match self.current_key.take() {
            None => None,
            Some(key) => {
                let v = mem::replace(&mut self.elts, Vec::new());
                Some((key, v))
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>)
    {
        let stored_count = self.current_key.is_some() as usize;
        let mut sh = size_hint::add_scalar(self.iter.size_hint(),
                                           stored_count);
        if sh.0 > 0 {
            sh.0 = 1;
        }
        sh
    }
}

/// An iterator adaptor that steps a number elements in the base iterator
/// for each iteration.
///
/// The iterator steps by yielding the next element from the base iterator,
/// then skipping forward *n-1* elements.
///
/// See [*.step()*](trait.Itertools.html#method.step) for more information.
#[derive(Clone)]
pub struct Step<I> {
    iter: Fuse<I>,
    skip: usize,
}

impl<I> Step<I> where I: Iterator
{
    /// Create a **Step** iterator.
    ///
    /// **Panics** if the step is 0.
    pub fn new(iter: I, step: usize) -> Self
    {
        assert!(step != 0);
        Step{iter: iter.fuse(), skip: step - 1}
    }
}

impl<I> Iterator for Step<I> where I: Iterator
{
    type Item = I::Item;
    #[inline]
    fn next(&mut self) -> Option<I::Item>
    {
        let elt = self.iter.next();
        self.iter.dropn(self.skip);
        elt
    }

    fn size_hint(&self) -> (usize, Option<usize>)
    {
        let (low, high) = self.iter.size_hint();
        let div = |x: usize| {
            if x == 0 {
                0
            } else {
                1 + (x - 1) / (self.skip + 1)
            }
        };
        (div(low), high.map(div))
    }
}

// known size
impl<I> ExactSizeIterator for Step<I> where
    I: ExactSizeIterator,
{ }

/// An iterator adaptor that merges the two base iterators in ascending order.
/// If both base iterators are sorted (ascending), the result is sorted.
///
/// Iterator element type is **I::Item**.
///
/// See [*.merge_by()*](trait.Itertools.html#method.merge_by) for more information.
pub struct Merge<I, J, F> where
    I: Iterator,
    J: Iterator<Item=I::Item>,
{
    a: Peekable<I>,
    b: Peekable<J>,
    cmp: F,
    fused: Option<bool>,
}

impl<I, J, F> Merge<I, J, F> where
    I: Iterator,
    J: Iterator<Item=I::Item>,
    F: FnMut(&I::Item, &I::Item) -> Ordering
{
    /// Create a **Merge** iterator.
    pub fn new(a: I, b: J, cmp: F) -> Self
    {
        Merge {
            a: a.peekable(),
            b: b.peekable(),
            cmp: cmp,
            fused: None,
        }
    }
}

impl<I, J, F> Clone for Merge<I, J, F> where
    I: Iterator,
    J: Iterator<Item=I::Item>,
    Peekable<I>: Clone,
    Peekable<J>: Clone,
    F: Clone,
{
    fn clone(&self) -> Self {
        clone_fields!(Merge, self, a, b, cmp, fused)
    }
}

impl<I, J, F> Iterator for Merge<I, J, F> where
    I: Iterator,
    J: Iterator<Item=I::Item>,
    F: FnMut(&I::Item, &I::Item) -> Ordering
{
    type Item = I::Item;

    fn next(&mut self) -> Option<I::Item> {
        let less_than = match self.fused {
            Some(lt) => lt,
            None => match (self.a.peek(), self.b.peek()) {
                (Some(a), Some(b)) => {
                    match (self.cmp)(a, b) {
                        Ordering::Less => true,
                        Ordering::Equal => true,
                        Ordering::Greater => false,
                    }
                }
                (Some(_), None) => {
                    self.fused = Some(true);
                    true
                }
                (None, Some(_)) => {
                    self.fused = Some(false);
                    false
                }
                (None, None) => return None,
            }
        };

        if less_than {
            self.a.next()
        } else {
            self.b.next()
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // Not ExactSizeIterator because size may be larger than usize
        size_hint::add(self.a.size_hint(), self.b.size_hint())
    }
}

#[cfg(feature = "unstable")]
/// An iterator adaptor that enumerates the iterator elements,
/// with a custom starting value and integer type.
///
/// See [*.enumerate_from()*](trait.Itertools.html#method.enumerate_from) for more information.
pub struct EnumerateFrom<I, K>
{
    index: K,
    iter: I,
}

#[cfg(feature = "unstable")]
impl<K, I> EnumerateFrom<I, K> where
    I: Iterator,
{
    /// Create a new **EnumerateFrom**.
    pub fn new(iter: I, start: K) -> Self
    {
        EnumerateFrom{index: start, iter: iter}
    }
}

#[cfg(feature = "unstable")]
impl<K, I> Iterator for EnumerateFrom<I, K> where
    K: Copy + One + Add<Output=K>,
    I: Iterator,
{
    type Item = (K, I::Item);
    fn next(&mut self) -> Option<(K, I::Item)>
    {
        match self.iter.next() {
            None => None,
            Some(elt) => {
                let index = self.index.clone();
                // FIXME: Arithmetic needs to be wrapping here to be sane,
                // imagine i8 counter to enumerate a sequence 0 to 127 inclusive.
                self.index = self.index + K::one();
                Some((index, elt))
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>)
    {
        self.iter.size_hint()
    }
}

// Same size
#[cfg(feature = "unstable")]
impl<K, I> ExactSizeIterator for EnumerateFrom<I, K> where
    K: Copy + One + Add<Output=K>,
    I: ExactSizeIterator,
{ }

#[derive(Clone)]
/// An iterator adaptor that allows the user to peek at multiple *.next()*
/// values without advancing itself.
///
/// See [*.multipeek()*](trait.Itertools.html#method.multipeek) for more information.
pub struct MultiPeek<I> where
    I: Iterator,
{
    iter: Fuse<I>,
    buf: Vec<I::Item>,
    index: usize,
}

impl<I: Iterator> MultiPeek<I> {
    /// Create a **MultiPeek** iterator.
    pub fn new(iter: I) -> MultiPeek<I> {
        MultiPeek{ iter: iter.fuse(), buf: Vec::new(), index: 0 }
    }

    /// Works exactly like *.next()* with the only difference that it doesn't
    /// advance itself. *.peek()* can be called multiple times, to peek
    /// further ahead.
    pub fn peek(&mut self) -> Option<&I::Item> {
        let ret = if self.index < self.buf.len() {
            Some(&self.buf[self.index])
        } else {
            match self.iter.next() {
                Some(x) => {
                    self.buf.push(x);
                    Some(&self.buf[self.index])
                }
                None => return None
            }
        };

        self.index += 1;
        ret
    }
}

impl<I> Iterator for MultiPeek<I> where
    I: Iterator,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<I::Item> {
        self.index = 0;
        if self.buf.is_empty() {
            self.iter.next()
        } else {
            Some(self.buf.remove(0))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>)
    {
        size_hint::add_scalar(self.iter.size_hint(), self.buf.len())
    }
}

// Same size
impl<I> ExactSizeIterator for MultiPeek<I> where
    I: ExactSizeIterator,
{ }

/// An iterator adaptor that may join together adjacent elements.
///
/// See [*.coalesce()*](trait.Itertools.html#method.coalesce) for more information.
#[derive(Clone)]
pub struct Coalesce<I, F> where
    I: Iterator,
{
    iter: I,
    last: Option<I::Item>,
    f: F,
}

/// An iterator adaptor that may join together adjacent elements.
pub type CoalesceFn<I> where I: Iterator =
    Coalesce<I, fn(I::Item, I::Item) -> Result<I::Item, (I::Item, I::Item)>>;

impl<I, F> Coalesce<I, F> where
    I: Iterator,
{
    /// Create a new **Coalesce**.
    pub fn new(mut iter: I, f: F) -> Self
    {
        Coalesce {
            last: iter.next(),
            iter: iter,
            f: f,
        }
    }
}

impl<I, F> Iterator for Coalesce<I, F> where
    I: Iterator,
    F: FnMut(I::Item, I::Item) -> Result<I::Item, (I::Item, I::Item)>
{
    type Item = I::Item;

    fn next(&mut self) -> Option<I::Item>
    {
        // this fuses the iterator
        let mut last = match self.last.take() {
            None => return None,
            Some(x) => x,
        };
        for next in &mut self.iter {
            match (self.f)(last, next) {
                Ok(joined) => last = joined,
                Err((last_, next_)) => {
                    self.last = Some(next_);
                    return Some(last_)
                }
            }
        }

        Some(last)
    }

    fn size_hint(&self) -> (usize, Option<usize>)
    {
        let (low, hi) = size_hint::add_scalar(self.iter.size_hint(),
                                              self.last.is_some() as usize);
        ((low > 0) as usize, hi)
    }
}



/// An iterator adaptor that borrows from a **Clone**-able iterator
/// to only pick off elements while the predicate returns **true**.
///
/// See [*.take_while_ref()*](trait.Itertools.html#method.take_while_ref) for more information.
pub struct TakeWhileRef<'a, I: 'a, F>
{
    iter: &'a mut I,
    f: F,
}

impl<'a, I, F> TakeWhileRef<'a, I, F> where I: Iterator + Clone,
{
    /// Create a new **TakeWhileRef** from a reference to clonable iterator.
    pub fn new(iter: &'a mut I, f: F) -> Self
    {
        TakeWhileRef {
            iter: iter,
            f: f,
        }
    }
}

impl<'a, I, F> Iterator for TakeWhileRef<'a, I, F> where
    I: Iterator + Clone,
    F: FnMut(&I::Item) -> bool,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<I::Item>
    {
        let old = self.iter.clone();
        match self.iter.next() {
            None => None,
            Some(elt) => {
                if (self.f)(&elt) {
                    Some(elt)
                } else {
                    *self.iter = old;
                    None
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>)
    {
        let (_, hi) = self.iter.size_hint();
        (0, hi)
    }
}

/// An iterator to iterate through all the combinations of pairs in a **Clone**-able iterator.
///
/// See [*.combinations()*](trait.Itertools.html#method.combinations) for more information.
#[derive(Clone)]
pub struct Combinations<I: Iterator> {
    iter: I,
    next_iter: I,
    val: Option<I::Item>,
}
impl<I> Combinations<I> where I: Iterator + Clone {
    /// Create a new **Combinations** from a clonable iterator.
    pub fn new(iter: I) -> Combinations<I> {
        Combinations { 
            next_iter: iter.clone(), 
            iter: iter, 
            val: None,
        }
    }
}

impl<I> Iterator for Combinations<I> where I: Iterator + Clone, I::Item: Clone{
    type Item = (I::Item, I::Item);
    fn next(&mut self) -> Option<Self::Item> {
        // not having a value means we iterate once more through the first iterator
        if self.val.is_none() {
            self.val = self.iter.next();
            self.next_iter = self.iter.clone();
        }

        // if its still none, we're out of values
        let elt = match self.val {
            Some(ref x) => x.clone(),
            None => return None,
        };

        match self.next_iter.next() {
            Some(ref x) => {
                return Some((elt, x.clone()));
            },
            None => {
                self.val = None;
            }
        }
        // try again if we ran out of values in the second iterator
        self.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lo, hi) = self.iter.size_hint();
        let (lo, hi) = size_hint::mul((lo, hi), (lo - 1, hi.map(|hi|hi - 1)));
        // won't truncate because x * (x - 1) is guarenteed to be even
        (lo / 2, hi.map(|hi| hi / 2))
    }
}
