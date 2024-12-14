use super::size_hint;
use std::cmp::Ordering;

/// An iterator which iterates two other iterators simultaneously
/// always returning elements are evenly sampled from the longest iterator.
///
/// See [`.zip_squash()`](crate::Itertools::zip_squash) for more information.
#[derive(Clone, Debug)]
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct ZipSquash<I: ExactSizeIterator, J: ExactSizeIterator> {
    a: I,
    b: J,
    a_delta: f32,
    b_delta: f32,
    a_index: f32,
    b_index: f32,
}

/// Zips two iterators skipping elements of the longest iterator to ensure it fully consumes both
/// iterators.
///
/// [`IntoIterator`] enabled version of [`Itertools::zip_squash`](crate::Itertools::zip_squash).
pub fn zip_squash<I, J>(i: I, j: J) -> ZipSquash<I::IntoIter, J::IntoIter>
where
    I: IntoIterator,
    J: IntoIterator,
    <I as IntoIterator>::IntoIter: ExactSizeIterator,
    <J as IntoIterator>::IntoIter: ExactSizeIterator,
{
    use std::iter::ExactSizeIterator;
    let (a, b) = (i.into_iter(), j.into_iter());
    let (a_delta, b_delta) = match a.len().cmp(&b.len()) {
        Ordering::Equal => (1f32, 1f32),
        Ordering::Less => (1f32, b.len() as f32 / a.len() as f32),
        Ordering::Greater => (a.len() as f32 / b.len() as f32, 1f32),
    };
    debug_assert!(a_delta >= 1f32);
    debug_assert!(b_delta >= 1f32);
    ZipSquash {
        a,
        b,
        a_delta,
        b_delta,
        a_index: 0f32,
        b_index: 0f32,
    }
}

impl<I, J> Iterator for ZipSquash<I, J>
where
    I: ExactSizeIterator,
    J: ExactSizeIterator,
{
    type Item = (I::Item, J::Item);

    fn next(&mut self) -> Option<Self::Item> {
        let (a, b) = (self.a.next(), self.b.next());
        let a_new = self.a_index + self.a_delta;
        if let Some(skip) = ((a_new.floor() - self.a_index.floor()) as usize).checked_sub(2) {
            self.a.nth(skip);
        }
        self.a_index = a_new;

        let b_new = self.b_index + self.b_delta;
        if let Some(skip) = ((b_new.floor() - self.b_index.floor()) as usize).checked_sub(2) {
            self.b.nth(skip);
        }
        self.b_index = b_new;

        a.zip(b)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        size_hint::min(self.a.size_hint(), self.b.size_hint())
    }
}

impl<I, J> ExactSizeIterator for ZipSquash<I, J>
where
    I: ExactSizeIterator,
    J: ExactSizeIterator,
{
}
