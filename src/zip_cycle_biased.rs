// ZipCycleBiased originally written by Aistis Raulinaitis

/// An iterator which iterates two other iterators simultaneously
/// and cycles the shorter iter until the longer is finished.
///
/// The bias is that if the left hand side iterator finishes, the entire iterator finishes,
/// even if the right hand side iterator has never cycled once.
///
/// This iterator is intended to be used when it is known beforehand which iterator will be shorter.
/// This allows a much faster and simpler implementation than `ZipCycle`, however it is less general.
///
/// See [`.zip_cycle_biased()`](../trait.Itertools.html#method.zip_cycle_biased) for more information.
#[derive(Debug, PartialEq, Clone)]
pub struct ZipCycleBiased<I, C>
    where I: Iterator,
          C: Clone+Iterator {
    i:      I,
    c_orig: C,
    c:      C
}

/// A `ZipCycleBiased` may return an error if the underlying cycle iterator is empty
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ZipCycleBiasedErr {
    /// The error result for when the Cycling half of the iter is empty
    CycleIterEmpty
}

pub fn zip_cycle_biased<I, C>(i: I, c: C) -> Result<ZipCycleBiased<I, C>, ZipCycleBiasedErr>
where I: Iterator,
      C: Clone + Iterator
{
    match c.clone().peekable().peek() {
        Some(_) => Ok(ZipCycleBiased { i, c_orig: c.clone(), c }),
        None => Err(ZipCycleBiasedErr::CycleIterEmpty),
    }
}

impl<I, C> Iterator for ZipCycleBiased<I, C>
    where I: Iterator,
          C: Clone+Iterator
{
    type Item = (<I as Iterator>::Item, <C as Iterator>::Item);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.i.next() {
            None => None,
            Some(i) => match self.c.next() {
                Some(c) => Some((i, c)),
                None => {
                    self.c = self.c_orig.clone();
                    Some((i, self.c.next().unwrap()))
                }
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.i.size_hint()
    }
}

// DoubleEndedIter todo.

impl<I, C> ExactSizeIterator for ZipCycleBiased<I, C>
    where I: ExactSizeIterator,
          C: ExactSizeIterator + Clone
{}
