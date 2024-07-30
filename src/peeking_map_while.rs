use crate::PutBack;
#[cfg(feature = "use_alloc")]
use crate::PutBackN;
use std::{fmt, iter::Peekable};

/// An iterator that allows peeking at an element before mapping it.
///
/// See [`.peeking_map_while()`](crate::Itertools::peeking_map_while)
/// for more information.
///
/// This is implemented by peeking adaptors like peekable and put back,
/// but also by a few iterators that can be peeked natively, like the slice’s
/// by reference iterator (`std::slice::Iter`).
pub trait PeekingMap<B>: Iterator {
    /// Type of the value returned by `predicate`.
    type MapItem;

    /// Passes a reference to the next iterator elemetn into the closure `predicate` to
    /// map to `B`. If either the `next()` call or `predicate()` returns None, iteration
    /// will stop.
    fn peeking_map<P>(&mut self, predicate: P) -> Option<B>
    where
        P: FnMut(&<Self as Iterator>::Item) -> Option<B>;
}

impl<I: Iterator, B> PeekingMap<B> for Peekable<I> {
    type MapItem = B;

    fn peeking_map<P>(&mut self, mut predicate: P) -> Option<B>
    where
        P: FnMut(&I::Item) -> Option<B>,
    {
        let x = self.peek()?;
        predicate(x).and_then(|x| {
            self.next();
            Some(x)
        })
    }
}

impl<I: Iterator, B> PeekingMap<B> for PutBack<I> {
    type MapItem = B;

    fn peeking_map<P>(&mut self, mut predicate: P) -> Option<B>
    where
        P: FnMut(&I::Item) -> Option<B>,
    {
        if let Some(x) = self.next() {
            predicate(&x).or_else(|| {
                self.put_back(x);
                None
            })
        } else {
            None
        }
    }
}

#[cfg(feature = "use_alloc")]
impl<I: Iterator, B> PeekingMap<B> for PutBackN<I> {
    type MapItem = B;

    fn peeking_map<P>(&mut self, mut predicate: P) -> Option<B>
    where
        P: FnMut(&Self::Item) -> Option<B>,
    {
        if let Some(x) = self.next() {
            predicate(&x).or_else(|| {
                self.put_back(x);
                None
            })
        } else {
            None
        }
    }
}

/// An iterator adaptor that only maps elements while `predicate` and `peek` returns `Some(_)`.
///
/// See [`.peeking_map_while()`](crate::Itertools::peeking_take_while)
/// for more information.
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct PeekingMapWhile<'iter, I, P>
where
    I: Iterator,
{
    iter: &'iter mut I,
    predicate: P,
}

impl<'iter, I, P> PeekingMapWhile<'iter, I, P>
where
    I: Iterator,
{
    /// Create a new `PeekingMapWhile` from an `Iterator` and `predicate`.
    pub fn new(iter: &'iter mut I, predicate: P) -> Self {
        PeekingMapWhile { iter, predicate }
    }
}

impl<'iter, I, P> fmt::Debug for PeekingMapWhile<'iter, I, P>
where
    I: fmt::Debug + Iterator,
    <I as Iterator>::Item: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PeekableMapWhile")
            .field("iter", &self.iter)
            .finish()
    }
}

impl<'iter, I, P, B> Iterator for PeekingMapWhile<'iter, I, P>
where
    I: PeekingMap<B>,
    P: FnMut(&I::Item) -> Option<B>,
{
    type Item = B;

    #[inline]
    fn next(&mut self) -> Option<B> {
        self.iter.peeking_map(&mut self.predicate)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.iter.size_hint();
        (0, upper) // can't know a lower bound, due to the predicate
    }
}

// Some iterators are so lightweight we can simply clone them to save their
// state and use that for peeking.
macro_rules! peeking_map_by_clone {
    ([$($typarm:tt)*] $type_:ty) => {
        impl<$($typarm)*, B> PeekingMap<B> for $type_
        {
            type MapItem = B;

            fn peeking_map<P>(&mut self, mut predicate: P) -> Option<B>
                where P: FnMut(&<$type_ as Iterator>::Item) -> Option<B>
            {
                let saved_state = self.clone();
                if let Some(r) = self.next() {
                    return if let Some(b) = predicate(&r) {
                        Some(b)
                    } else {
                        *self = saved_state;
                        None
                    }
                }
                None
            }
        }
    }
}

peeking_map_by_clone! { ['a, T] ::std::slice::Iter<'a, T> }
peeking_map_by_clone! { ['a] ::std::str::Chars<'a> }
peeking_map_by_clone! { ['a] ::std::str::CharIndices<'a> }
peeking_map_by_clone! { ['a] ::std::str::Bytes<'a> }
peeking_map_by_clone! { ['a, T] ::std::option::Iter<'a, T> }
peeking_map_by_clone! { ['a, T] ::std::result::Iter<'a, T> }
peeking_map_by_clone! { [T] ::std::iter::Empty<T> }
#[cfg(feature = "use_alloc")]
peeking_map_by_clone! { ['a, T] alloc::collections::linked_list::Iter<'a, T> }
#[cfg(feature = "use_alloc")]
peeking_map_by_clone! { ['a, T] alloc::collections::vec_deque::Iter<'a, T> }

// cloning a Rev has no extra overhead; peekable and put backs are never DEI.
peeking_map_by_clone! { [I: Clone + PeekingMap<B> + DoubleEndedIterator] ::std::iter::Rev<I> }
