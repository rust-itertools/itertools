use crate::size_hint;
use std::{
    fmt,
    iter::{DoubleEndedIterator, FusedIterator},
};

pub fn flat_map_ok<I, F, T, U, E>(iter: I, f: F) -> FlatMapOk<I, F, T, U, E>
where
    I: Iterator<Item = Result<T, E>>,
    F: FnMut(T) -> U,
    U: IntoIterator,
{
    FlatMapOk {
        iter,
        f,
        inner_front: None,
        inner_back: None,
        _phantom: std::marker::PhantomData,
    }
}

/// An iterator adaptor that applies a function to `Result::Ok` values and
/// flattens the resulting iterator. `Result::Err` values are passed through
/// unchanged.
///
/// This is equivalent to `.map_ok(f).flatten_ok()`.
///
/// See [`.flat_map_ok()`](crate::Itertools::flat_map_ok) for more information.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct FlatMapOk<I, F, T, U, E>
where
    I: Iterator<Item = Result<T, E>>,
    F: FnMut(T) -> U,
    U: IntoIterator,
{
    iter: I,
    f: F,
    inner_front: Option<U::IntoIter>,
    inner_back: Option<U::IntoIter>,
    _phantom: std::marker::PhantomData<fn() -> (T, E)>,
}

impl<I, F, T, U, E> Iterator for FlatMapOk<I, F, T, U, E>
where
    I: Iterator<Item = Result<T, E>>,
    F: FnMut(T) -> U,
    U: IntoIterator,
{
    type Item = Result<U::Item, E>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(inner) = &mut self.inner_front {
                if let Some(item) = inner.next() {
                    return Some(Ok(item));
                }
                self.inner_front = None;
            }

            match self.iter.next() {
                Some(Ok(ok)) => self.inner_front = Some((self.f)(ok).into_iter()),
                Some(Err(e)) => return Some(Err(e)),
                None => {
                    if let Some(inner) = &mut self.inner_back {
                        if let Some(item) = inner.next() {
                            return Some(Ok(item));
                        }
                        self.inner_back = None;
                    } else {
                        return None;
                    }
                }
            }
        }
    }

    fn fold<B, Fold>(self, init: B, mut fold_f: Fold) -> B
    where
        Self: Sized,
        Fold: FnMut(B, Self::Item) -> B,
    {
        let mut f = self.f;

        // Front
        let mut acc = match self.inner_front {
            Some(x) => x.fold(init, |a, o| fold_f(a, Ok(o))),
            None => init,
        };

        acc = self.iter.fold(acc, |acc, x| match x {
            Ok(ok) => f(ok).into_iter().fold(acc, |a, o| fold_f(a, Ok(o))),
            Err(e) => fold_f(acc, Err(e)),
        });

        // Back
        match self.inner_back {
            Some(x) => x.fold(acc, |a, o| fold_f(a, Ok(o))),
            None => acc,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let inner_hint = |inner: &Option<U::IntoIter>| {
            inner
                .as_ref()
                .map(Iterator::size_hint)
                .unwrap_or((0, Some(0)))
        };
        let inner_front = inner_hint(&self.inner_front);
        let inner_back = inner_hint(&self.inner_back);
        let outer = match self.iter.size_hint() {
            (0, Some(0)) => (0, Some(0)),
            _ => (0, None),
        };

        size_hint::add(size_hint::add(inner_front, inner_back), outer)
    }
}

impl<I, F, T, U, E> DoubleEndedIterator for FlatMapOk<I, F, T, U, E>
where
    I: DoubleEndedIterator<Item = Result<T, E>>,
    F: FnMut(T) -> U,
    U: IntoIterator,
    U::IntoIter: DoubleEndedIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(inner) = &mut self.inner_back {
                if let Some(item) = inner.next_back() {
                    return Some(Ok(item));
                }
                self.inner_back = None;
            }

            match self.iter.next_back() {
                Some(Ok(ok)) => self.inner_back = Some((self.f)(ok).into_iter()),
                Some(Err(e)) => return Some(Err(e)),
                None => {
                    if let Some(inner) = &mut self.inner_front {
                        if let Some(item) = inner.next_back() {
                            return Some(Ok(item));
                        }
                        self.inner_front = None;
                    } else {
                        return None;
                    }
                }
            }
        }
    }

    fn rfold<B, Fold>(self, init: B, mut fold_f: Fold) -> B
    where
        Self: Sized,
        Fold: FnMut(B, Self::Item) -> B,
    {
        let mut f = self.f;

        // Back
        let mut acc = match self.inner_back {
            Some(x) => x.rfold(init, |a, o| fold_f(a, Ok(o))),
            None => init,
        };

        acc = self.iter.rfold(acc, |acc, x| match x {
            Ok(ok) => f(ok).into_iter().rfold(acc, |a, o| fold_f(a, Ok(o))),
            Err(e) => fold_f(acc, Err(e)),
        });

        // Front
        match self.inner_front {
            Some(x) => x.rfold(acc, |a, o| fold_f(a, Ok(o))),
            None => acc,
        }
    }
}

impl<I, F, T, U, E> Clone for FlatMapOk<I, F, T, U, E>
where
    I: Iterator<Item = Result<T, E>> + Clone,
    F: FnMut(T) -> U + Clone,
    U: IntoIterator,
    U::IntoIter: Clone,
{
    clone_fields!(iter, f, inner_front, inner_back, _phantom);
}

impl<I, F, T, U, E> fmt::Debug for FlatMapOk<I, F, T, U, E>
where
    I: Iterator<Item = Result<T, E>> + fmt::Debug,
    F: FnMut(T) -> U,
    U: IntoIterator,
    U::IntoIter: fmt::Debug,
{
    debug_fmt_fields!(FlatMapOk, iter, inner_front, inner_back);
}

/// Only the iterator being flat-mapped needs to implement [`FusedIterator`].
impl<I, F, T, U, E> FusedIterator for FlatMapOk<I, F, T, U, E>
where
    I: FusedIterator<Item = Result<T, E>>,
    F: FnMut(T) -> U,
    U: IntoIterator,
{
}
