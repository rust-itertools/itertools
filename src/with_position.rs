use std::fmt;
use std::iter::{Fuse, FusedIterator, Peekable};

/// An iterator adaptor that wraps each element in an [`Position`].
///
/// Iterator element type is `(Position, I::Item)`.
///
/// See [`.with_position()`](crate::Itertools::with_position) for more information.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct WithPosition<I>
where
    I: Iterator,
{
    handled_first: bool,
    peekable: Peekable<Fuse<I>>,
}

impl<I> fmt::Debug for WithPosition<I>
where
    I: Iterator,
    Peekable<Fuse<I>>: fmt::Debug,
{
    debug_fmt_fields!(WithPosition, handled_first, peekable);
}

impl<I> Clone for WithPosition<I>
where
    I: Clone + Iterator,
    I::Item: Clone,
{
    clone_fields!(handled_first, peekable);
}

/// Create a new `WithPosition` iterator.
pub fn with_position<I>(iter: I) -> WithPosition<I>
where
    I: Iterator,
{
    WithPosition {
        handled_first: false,
        peekable: iter.fuse().peekable(),
    }
}

/// The first component of the value yielded by `WithPosition`.
/// Indicates the position of this element in the iterator results.
///
/// See [`.with_position()`](crate::Itertools::with_position) for more information.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Position {
    /// This is the initial element (also true if there's exactly one element)
    pub is_first: bool,
    /// This is the final element (also true if there's exactly one element)
    pub is_last: bool,
}

impl Position {
    /// This is the first and the last element at the same time, and there are no more elements
    pub fn is_exactly_one(self) -> bool {
        self.is_first && self.is_last
    }

    /// This is neither first nor last element, and there will be more elements
    pub fn is_middle(self) -> bool {
        !self.is_first && !self.is_last
    }

    /// This is the initial element (also true if there's exactly one element)
    pub fn is_first(self) -> bool {
        self.is_first
    }

    /// This is the final element (also true if there's exactly one element)
    pub fn is_last(self) -> bool {
        self.is_last
    }
}

impl<I: Iterator> Iterator for WithPosition<I> {
    type Item = (Position, I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.peekable.next()?;

        let is_last = self.peekable.peek().is_none();
        let is_first = !self.handled_first;
        self.handled_first = true;

        Some((Position { is_first, is_last }, item))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.peekable.size_hint()
    }

    fn fold<B, F>(mut self, mut init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        if let Some(mut head) = self.peekable.next() {
            if !self.handled_first {
                // The current head is `First` or `Only`,
                // it depends if there is another item or not.
                match self.peekable.next() {
                    Some(second) => {
                        let first = std::mem::replace(&mut head, second);
                        let position = Position {
                            is_first: true,
                            is_last: false,
                        };
                        init = f(init, (position, first));
                    }
                    None => {
                        let position = Position {
                            is_first: true,
                            is_last: true,
                        };
                        return f(init, (position, head));
                    }
                }
            }
            // Have seen the first item, and there's something left.
            init = self.peekable.fold(init, |acc, mut item| {
                std::mem::swap(&mut head, &mut item);
                let position = Position {
                    is_first: false,
                    is_last: false,
                };
                f(acc, (position, item))
            });
            let position = Position {
                is_first: false,
                is_last: true,
            };
            // The "head" is now the last item.
            init = f(init, (position, head));
        }
        init
    }
}

impl<I> ExactSizeIterator for WithPosition<I> where I: ExactSizeIterator {}

impl<I: Iterator> FusedIterator for WithPosition<I> {}
