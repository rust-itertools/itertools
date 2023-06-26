use crate::size_hint;
use core::iter::ExactSizeIterator;
use core::mem;

/// The enum returned by `at_most_one`, depending on the number of remaining
/// elements in the iterator.
/// 
/// See [`.at_most_one()`](crate::Itertools::at_most_one) for more detail.
#[derive(PartialEq, Eq, Clone)]
pub enum AtMostOneResult<I: Iterator> {
    /// The iterator was empty and therefore had zero elements.
    Zero,

    /// The iterator had exactly one element.
    One(I::Item),

    /// The iterator had more than one element. 
    /// `MoreThanOne` is an iterator which yields the same elements as the original iterator.
    MoreThanOne(MoreThanOne<I>),
}

#[derive(PartialEq, Eq, Clone)]
enum IterSource<T> {
    FirstElement(T, T),
    SecondElement(T),
    InnerIter,
}

/// The iterator returned by [`.at_most_one()`](crate::Itertools::at_most_one), if the original iterator
/// had at least two elements remaining. Yields the same elements as the original iterator.
#[derive(PartialEq, Eq, Clone)]
pub struct MoreThanOne<I: Iterator> {
    next_source: IterSource<I::Item>,
    inner: I,
}

impl<I: Iterator> MoreThanOne<I> {
    pub(crate) fn new(first_two: [I::Item; 2], inner: I) -> Self {
        let [first, second] = first_two;
        let next_source = IterSource::FirstElement(first, second);

        Self { next_source, inner }
    }

    fn additional_len(&self) -> usize {
        match self.next_source {
            IterSource::FirstElement(_, _) => 2,
            IterSource::SecondElement(_) => 1,
            IterSource::InnerIter => 0,
        }
    }
}

impl<I: Iterator> Iterator for MoreThanOne<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let source = mem::replace(&mut self.next_source, IterSource::InnerIter);

        match source {
            IterSource::FirstElement(first, second) => {
                self.next_source = IterSource::SecondElement(second);
                Some(first)
            }
            IterSource::SecondElement(second) => Some(second),
            IterSource::InnerIter => self.inner.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        size_hint::add_scalar(self.inner.size_hint(), self.additional_len())
    }
}

impl<I> ExactSizeIterator for MoreThanOne<I> where I: ExactSizeIterator {}
