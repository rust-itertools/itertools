use core::{iter::Peekable, mem};

use either::Either;

use crate::PutBack;

/// An iterator adaptor that takes items while a closure returns [`Either::Left`].
///
/// See [`.peeking_map_while()`](crate::Itertools::peeking_map_while)
/// for more information.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct PeekingMapWhile<'a, I, F> {
    pub(crate) iter: &'a mut I,
    pub(crate) accept: F,
}

impl<I, F, O> Iterator for PeekingMapWhile<'_, I, F>
where
    I: PeekingMapNext,
    F: FnMut(I::Item) -> Either<O, I::Item>,
{
    type Item = O;

    fn next(&mut self) -> Option<O> {
        self.iter.peeking_map_next(&mut self.accept)
    }
}

/// A trait used in [`PeekingMapWhile`]
pub trait PeekingMapNext: Iterator {
    /// If it returns [`Either::Right`] then it should be reinserted to the iterator otherwise it should be returned
    fn peeking_map_next<O>(
        &mut self,
        accept: impl FnMut(Self::Item) -> Either<O, Self::Item>,
    ) -> Option<O>;
}

impl<I> PeekingMapNext for Peekable<I>
where
    I: Iterator,
    I::Item: Default,
{
    fn peeking_map_next<O>(
        &mut self,
        mut accept: impl FnMut(I::Item) -> Either<O, I::Item>,
    ) -> Option<O> {
        let dest = self.peek_mut()?;
        let item = mem::take(dest);
        let out = accept(item);
        match out {
            Either::Left(out) => {
                self.next();
                Some(out)
            }
            Either::Right(item) => {
                *dest = item;
                None
            }
        }
    }
}

impl<I> PeekingMapNext for PutBack<I>
where
    I: Iterator,
{
    fn peeking_map_next<O>(
        &mut self,
        mut accept: impl FnMut(I::Item) -> Either<O, I::Item>,
    ) -> Option<O> {
        match accept(self.next()?) {
            Either::Left(out) => Some(out),
            Either::Right(item) => {
                self.put_back(item);
                None
            }
        }
    }
}

impl<'a, I, F> std::fmt::Debug for PeekingMapWhile<'a, I, F>
where
    I: Iterator + std::fmt::Debug + 'a,
{
    debug_fmt_fields!(PeekingTakeWhile, iter);
}
