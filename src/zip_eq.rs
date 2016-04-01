use super::size_hint;

/// An iterator which iterates two other iterators simultaneously
///
/// See [*.zip_eq()*](trait.Itertools.html#method.zip_eq) for more information.
#[derive(Clone)]
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct ZipEq<I, J> {
    a: I,
    b: J,
}

pub fn new<I, J>(a: I, b: J) -> ZipEq<I, J> {
    ZipEq {
        a: a,
        b: b,
    }
}

impl<I, J> Iterator for ZipEq<I, J>
    where I: Iterator,
          J: Iterator
{
    type Item = (I::Item, J::Item);

    fn next(&mut self) -> Option<Self::Item> {
        match (self.a.next(), self.b.next()) {
            (None, None) => None,
            (Some(a), Some(b)) => Some((a, b)),
            (None, Some(_)) | (Some(_), None) =>
            panic!("itertools: .zip_eq() reached end of one iterator before the other")
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        size_hint::min(self.a.size_hint(), self.b.size_hint())
    }
}

impl<I, J> ExactSizeIterator for ZipEq<I, J>
    where I: ExactSizeIterator,
          J: ExactSizeIterator
{}
