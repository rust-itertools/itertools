use std::cmp;
use std::iter::RandomAccessIterator;
use self::EitherOrBoth::{Right, Left, Both};

// ZipLongest originally written by SimonSapin,
// and dedicated to itertools https://github.com/rust-lang/rust/pull/19283

/// An iterator which iterates two other iterators simultaneously
#[derive(Clone)]
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct ZipLongest<T, U> {
    a: T,
    b: U
}

impl<T, U> ZipLongest<T, U>
{
    /// Create a new ZipLongest iterator.
    pub fn new(a: T, b: U) -> ZipLongest<T, U>
    {
        ZipLongest{a: a, b: b}
    }
}

impl<A, B, T, U> Iterator for ZipLongest<T, U>
    where
        T: Iterator<Item=A>,
        U: Iterator<Item=B>,
{
    type Item = EitherOrBoth<A, B>;
    #[inline]
    fn next(&mut self) -> Option<EitherOrBoth<A, B>> {
        match (self.a.next(), self.b.next()) {
            (None, None) => None,
            (Some(a), None) => Some(Left(a)),
            (None, Some(b)) => Some(Right(b)),
            (Some(a), Some(b)) => Some(Both(a, b)),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (a_lower, a_upper) = self.a.size_hint();
        let (b_lower, b_upper) = self.b.size_hint();

        let lower = cmp::max(a_lower, b_lower);

        let upper = match (a_upper, b_upper) {
            (Some(x), Some(y)) => Some(cmp::max(x,y)),
            _ => None
        };

        (lower, upper)
    }
}

impl<A, B, T, U> DoubleEndedIterator for ZipLongest<T, U>
    where
        T: DoubleEndedIterator<Item=A> + ExactSizeIterator,
        U: DoubleEndedIterator<Item=B> + ExactSizeIterator,
{
    #[inline]
    fn next_back(&mut self) -> Option<EitherOrBoth<A, B>> {
        use std::cmp::Ordering::{Equal, Greater, Less};
        match self.a.len().cmp(&self.b.len()) {
            Equal => match (self.a.next_back(), self.b.next_back()) {
                (None, None) => None,
                (Some(a), Some(b)) => Some(Both(a, b)),
                // These can only happen if .len() is inconsistent with .next_back()
                (Some(a), None) => Some(Left(a)),
                (None, Some(b)) => Some(Right(b)),
            },
            Greater => self.a.next_back().map(Left),
            Less => self.b.next_back().map(Right),
        }
    }
}

impl<A, B, T, U> RandomAccessIterator for ZipLongest<T, U>
    where
        T: RandomAccessIterator<Item=A>,
        U: RandomAccessIterator<Item=B>,
{
    #[inline]
    fn indexable(&self) -> usize {
        cmp::max(self.a.indexable(), self.b.indexable())
    }

    #[inline]
    fn idx(&mut self, index: usize) -> Option<EitherOrBoth<A, B>> {
        match (self.a.idx(index), self.b.idx(index)) {
            (None, None) => None,
            (Some(a), None) => Some(Left(a)),
            (None, Some(b)) => Some(Right(b)),
            (Some(a), Some(b)) => Some(Both(a, b)),
        }
    }
}

#[unstable = "trait is unstable"]
impl<T, U> ExactSizeIterator for ZipLongest<T, U>
    where T: ExactSizeIterator, U: ExactSizeIterator {}


/// A value yielded by `ZipLongest`.
/// Contains one or two values,
/// depending on which of the input iterators are exhausted.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum EitherOrBoth<A, B> {
    /// Neither input iterator is exhausted yet, yielding two values.
    Both(A, B),
    /// The parameter iterator of `.zip_longest()` is exhausted,
    /// only yielding a value from the `self` iterator.
    Left(A),
    /// The `self` iterator of `.zip_longest()` is exhausted,
    /// only yielding a value from the parameter iterator.
    Right(B),
}
