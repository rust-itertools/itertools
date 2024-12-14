use super::size_hint;
use std::cmp::Ordering;
use std::fmt;

/// An iterator which iterates two other iterators simultaneously
/// always returning the first and last elements of both iterators by using
/// cloning to extend the length of the shortest iterator.
///
/// See [`.zip_stretch()`](crate::Itertools::zip_stretch) for more information.
#[derive(Clone)]
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct ZipStretch<I: ExactSizeIterator, J: ExactSizeIterator>
where
    <I as Iterator>::Item: Clone,
    <J as Iterator>::Item: Clone,
{
    a: I,
    b: J,
    a_delta: f32,
    b_delta: f32,
    a_index: f32,
    b_index: f32,
    a_dupe: Option<<I as Iterator>::Item>,
    b_dupe: Option<<J as Iterator>::Item>,
}

impl<I: ExactSizeIterator + fmt::Debug, J: ExactSizeIterator + fmt::Debug> fmt::Debug
    for ZipStretch<I, J>
where
    <I as Iterator>::Item: Clone,
    <J as Iterator>::Item: Clone,
{
    debug_fmt_fields!(ZipStretch, a, b, a_delta, b_delta, a_index, b_index);
}

/// Zips two iterators cloning elements to extend the length of the shortest iterator to
/// ensure it fully consumes both iterators.
///
/// [`IntoIterator`] enabled version of [`Itertools::zip_stretch`](crate::Itertools::zip_stretch).
pub fn zip_stretch<I, J>(i: I, j: J) -> ZipStretch<I::IntoIter, J::IntoIter>
where
    I: IntoIterator,
    J: IntoIterator,
    <I as IntoIterator>::IntoIter: ExactSizeIterator,
    <J as IntoIterator>::IntoIter: ExactSizeIterator,
    <<I as IntoIterator>::IntoIter as IntoIterator>::Item: Clone,
    <<J as IntoIterator>::IntoIter as IntoIterator>::Item: Clone,
{
    use std::iter::ExactSizeIterator;
    let (a, b) = (i.into_iter(), j.into_iter());
    let (a_delta, b_delta) = match a.len().cmp(&b.len()) {
        Ordering::Equal => (1f32, 1f32),
        Ordering::Less => (a.len() as f32 / b.len() as f32, 1f32),
        Ordering::Greater => (1f32, b.len() as f32 / a.len() as f32),
    };
    debug_assert!(a_delta <= 1f32);
    debug_assert!(b_delta <= 1f32);
    ZipStretch {
        a,
        b,
        a_delta,
        b_delta,
        a_index: 0f32,
        b_index: 0f32,
        a_dupe: None,
        b_dupe: None,
    }
}

impl<I, J> Iterator for ZipStretch<I, J>
where
    I: ExactSizeIterator,
    J: ExactSizeIterator,
    <I as Iterator>::Item: Clone,
    <J as Iterator>::Item: Clone,
{
    type Item = (I::Item, J::Item);

    fn next(&mut self) -> Option<Self::Item> {
        if self.a_index.fract() < self.a_delta {
            self.a_dupe = self.a.next();
        }
        self.a_index += self.a_delta;

        if self.b_index.fract() < self.b_delta {
            self.b_dupe = self.b.next();
        }
        self.b_index += self.b_delta;

        self.a_dupe.clone().zip(self.b_dupe.clone())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        size_hint::min(self.a.size_hint(), self.b.size_hint())
    }
}

impl<I, J> ExactSizeIterator for ZipStretch<I, J>
where
    I: ExactSizeIterator,
    J: ExactSizeIterator,
    <I as Iterator>::Item: Clone,
    <J as Iterator>::Item: Clone,
{
}
