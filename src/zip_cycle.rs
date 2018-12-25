use super::size_hint;

// ZipCycle originally written by Aistis Raulinaitis

/// An iterator which iterates two other iterators simultaneously
/// and cycles the shorter iter until the longer is finished.
///
/// See [`.zip_cycle()`](../trait.Itertools.html#method.zip_cycle) for more information.
#[derive(Debug, PartialEq, Clone)]
pub struct ZipCycle<A, B>
where
    A: Clone + Iterator,
    B: Clone + Iterator,
{
    ab: Option<AB>,
    a_orig: A,
    a: A,
    b_orig: B,
    b: B,
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum AB {
    A,
    B,
}

/// A `ZipCycle` on construction with `ZipCycle::new()` may end up in 3 error states, all having
/// to do with either one or both of the underlying iterators being empty.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ZipCycleErr {
    /// The case both are empty
    BothItersEmpty,
    /// The case the left hand side is empty
    FirstIterEmpty,
    /// The case the right hand side is empty
    SecondIterEmpty
}

pub fn zip_cycle<A, B>(a: A, b: B) -> Result<ZipCycle<A, B>, ZipCycleErr>
where
    A: Clone + Iterator,
    B: Clone + Iterator,
{
    match (a.clone().peekable().peek(), b.clone().peekable().peek()) {
        (Some(_), Some(_)) => Ok(ZipCycle {
            ab: None,
            a_orig: a.clone(),
            a,
            b_orig: b.clone(),
            b,
        }),
        (None, None) => Err(ZipCycleErr::BothItersEmpty),
        (None, Some(_)) => Err(ZipCycleErr::FirstIterEmpty),
        (Some(_), None) => Err(ZipCycleErr::SecondIterEmpty),
    }
}

impl<A, B> Iterator for ZipCycle<A, B>
where
    A: Clone + Iterator,
    B: Clone + Iterator,
{
    type Item = (
        <A as Iterator>::Item,
        <B as Iterator>::Item,
    );

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Inner unwraps are innocuous because of checking in `ZipCycle::new()`
        match self.ab {
            None => match (self.a.next(), self.b.next()) {
                (None, None) => None,
                (Some(a), Some(b)) => Some((a, b)),
                (None, Some(b)) => {
                    self.ab = Some(AB::B);
                    self.a = self.a_orig.clone();
                    Some((self.a.next().unwrap(), b))
                }
                (Some(a), None) => {
                    self.ab = Some(AB::A);
                    self.b = self.b_orig.clone();
                    Some((a, self.b.next().unwrap()))
                }
            },
            Some(ref ab) => match (self.a.next(), self.b.next()) {
                (None, None) => None,
                (Some(a), Some(b)) => Some((a, b)),
                (None, Some(b)) => match *ab {
                    AB::A => None,
                    AB::B => {
                        self.a = self.a_orig.clone();
                        Some((self.a.next().unwrap(), b))
                    }
                },
                (Some(a), None) => match *ab {
                    AB::B => None,
                    AB::A => {
                        self.b = self.b_orig.clone();
                        Some((a, self.b.next().unwrap()))
                    }
                },
            },
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        size_hint::max(self.a.size_hint(), self.b.size_hint())
    }
}

// DoubleEndedIter todo.

impl<A, B> ExactSizeIterator for ZipCycle<A, B>
    where A: ExactSizeIterator + Clone,
          B: ExactSizeIterator + Clone
{}