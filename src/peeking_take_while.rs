
use std::iter::Peekable;
use PutBack;
use PutBackN;

/// An iterator that allows peeking at an element before deciding
/// to accept it.
///
/// See [`.peeking_take_while()`](trait.Itertools.html#method.peeking_take_while)
/// for more information.
pub trait PeekingNext : Iterator {
    /// Pass a reference to the next iterator element to the closure `accept`;
    /// if `accept` returns true, return it as the next element,
    /// else None.
    fn peeking_next<F>(&mut self, accept: F) -> Option<Self::Item>
        where F: FnOnce(&Self::Item) -> bool;
}

impl<I> PeekingNext for Peekable<I>
    where I: Iterator,
{
    fn peeking_next<F>(&mut self, accept: F) -> Option<Self::Item>
        where F: FnOnce(&Self::Item) -> bool
    {
        if let Some(r) = self.peek() {
            if !accept(r) {
                return None;
            }
        }
        self.next()
    }
}

impl<I> PeekingNext for PutBack<I>
    where I: Iterator,
{
    fn peeking_next<F>(&mut self, accept: F) -> Option<Self::Item>
        where F: FnOnce(&Self::Item) -> bool
    {
        if let Some(r) = self.next() {
            if !accept(&r) {
                self.put_back(r);
                return None;
            }
            Some(r)
        } else {
            None
        }
    }
}

impl<I> PeekingNext for PutBackN<I>
    where I: Iterator,
{
    fn peeking_next<F>(&mut self, accept: F) -> Option<Self::Item>
        where F: FnOnce(&Self::Item) -> bool
    {
        if let Some(r) = self.next() {
            if !accept(&r) {
                self.put_back(r);
                return None;
            }
            Some(r)
        } else {
            None
        }
    }
}

/// An iterator adaptor that takes items while a closure returns `true`.
///
/// See [`.peeking_take_while()`](../trait.Itertools.html#method.peeking_take_while)
/// for more information.
pub struct PeekingTakeWhile<'a, I: 'a, F>
    where I: Iterator,
{
    iter: &'a mut I,
    f: F,
}

/// Create a PeekingTakeWhile
pub fn peeking_take_while<I, F>(iter: &mut I, f: F) -> PeekingTakeWhile<I, F>
    where I: Iterator,
{
    PeekingTakeWhile {
        iter: iter,
        f: f,
    }
}

impl<'a, I, F> Iterator for PeekingTakeWhile<'a, I, F>
    where I: PeekingNext,
          F: FnMut(&I::Item) -> bool,

{
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.peeking_next(&mut self.f)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, hi) = self.iter.size_hint();
        (0, hi)
    }
}

