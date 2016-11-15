use std::fmt;
use std::iter::Peekable;

/// An iterator adaptor that wraps each element in an [`Position`](../enum.Position.html).
///
/// Iterator element type is `Position<I::Item>`.
/// This iterator is *fused*.
///
/// See [`.note_position()`](../trait.Itertools.html#method.note_position) for more information.
pub struct NotePosition<I>
    where I: Iterator,
{
    handled_first: bool,
    peekable: Peekable<I>,
}

/// Create a new `NotePosition` iterator.
pub fn note_position<I>(iter: I) -> NotePosition<I>
    where I: Iterator,
{
    NotePosition {
        handled_first: false,
        peekable: iter.peekable(),
    }
}

/// A value yielded by `NotePosition`.
/// Indicates the position of this element in the iterator results.
///
/// See [`.note_position()`](trait.Itertools.html#method.note_position) for more information.
pub enum Position<T> {
    /// This is the first element.
    First(T),
    /// This is neither the first nor the last element.
    Middle(T),
    /// This is the last element.
    Last(T),
    /// This is the only element.
    Only(T),
}

impl<T: PartialEq> PartialEq for Position<T> {
    fn eq(&self, other: &Position<T>) -> bool {
        match (self, other) {
            (&Position::First(ref a), &Position::First(ref b)) => a == b,
            (&Position::Middle(ref a), &Position::Middle(ref b)) => a == b,
            (&Position::Last(ref a), &Position::Last(ref b)) => a == b,
            (&Position::Only(ref a), &Position::Only(ref b)) => a == b,
            _ => false,
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Position<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Position::First(ref a) => write!(f, "First({:?})", a),
            &Position::Middle(ref a) => write!(f, "Middle({:?})", a),
            &Position::Last(ref a) => write!(f, "Last({:?})", a),
            &Position::Only(ref a) => write!(f, "Only({:?})", a),
        }
    }
}

impl<I: Iterator> Iterator for NotePosition<I> {
    type Item = Position<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        match (self.handled_first, self.peekable.next()) {
            // Haven't seen the first item yet, and there is one to give.
            (false, Some(item)) => {
                self.handled_first = true;
                // Peek to see if this is also the last item,
                // in which case tag it as `Only`.
                match self.peekable.peek() {
                    Some(_) => Some(Position::First(item)),
                    None => Some(Position::Only(item)),
                }
            }
            // Have seen the first item, and there's something left.
            (true, Some(item)) => {
                // Peek to see if this is the last item.
                match self.peekable.peek() {
                    Some(_) => Some(Position::Middle(item)),
                    None => Some(Position::Last(item)),
                }
            }
            // Iterator is finished.
            (_, None) => None,
        }
    }
}
