use std::iter::{Fuse, FusedIterator};

/// An iterator adaptor that pads a sequence to a minimum length by filling
/// missing elements using a closure.
///
/// Iterator element type is `I::Item`.
///
/// See [`.pad_using()`](crate::Itertools::pad_using) for more information.
#[derive(Clone)]
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct PadUsing<I, F> {
    iter: Fuse<I>,
    elements_from_next: usize,
    elements_from_next_back: usize,
    elements_required: usize,
    filler: F,
}

impl<I, F> std::fmt::Debug for PadUsing<I, F>
where
    I: std::fmt::Debug,
{
    debug_fmt_fields!(
        PadUsing,
        iter,
        elements_from_next,
        elements_from_next_back,
        elements_required
    );
}

/// Create a new `PadUsing` iterator.
pub fn pad_using<I, F>(iter: I, elements_required: usize, filler: F) -> PadUsing<I, F>
where
    I: Iterator,
    F: FnMut(usize) -> I::Item,
{
    PadUsing {
        iter: iter.fuse(),
        elements_from_next: 0,
        elements_from_next_back: 0,
        elements_required,
        filler,
    }
}

impl<I, F> Iterator for PadUsing<I, F>
where
    I: Iterator,
    F: FnMut(usize) -> I::Item,
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let total_consumed = self.elements_from_next + self.elements_from_next_back;

        if total_consumed >= self.elements_required {
            self.iter.next()
        } else if let Some(e) = self.iter.next() {
            self.elements_from_next += 1;
            Some(e)
        } else {
            let e = (self.filler)(self.elements_from_next);
            self.elements_from_next += 1;
            Some(e)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let total_consumed = self.elements_from_next + self.elements_from_next_back;

        if total_consumed >= self.elements_required {
            return self.iter.size_hint();
        }

        let elements_remaining = self.elements_required - total_consumed;
        let (low, high) = self.iter.size_hint();

        let lower_bound = low.max(elements_remaining);
        let upper_bound = high.map(|h| h.max(elements_remaining));

        (lower_bound, upper_bound)
    }
}

impl<I, F> DoubleEndedIterator for PadUsing<I, F>
where
    I: DoubleEndedIterator + ExactSizeIterator,
    F: FnMut(usize) -> I::Item,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let total_consumed = self.elements_from_next + self.elements_from_next_back;

        if total_consumed >= self.elements_required {
            return self.iter.next_back();
        }

        let elements_remaining = self.elements_required - total_consumed;
        self.elements_from_next_back += 1;

        if self.iter.len() < elements_remaining {
            Some((self.filler)(
                self.elements_required - self.elements_from_next_back,
            ))
        } else {
            let item = self.iter.next_back();
            debug_assert!(item.is_some()); // If this triggers, we should not have incremented elements_from_next_back, and the input iterator mistakenly reported that it would be able to produce at least elements_remaining items.
            item
        }
    }
}

impl<I, F> ExactSizeIterator for PadUsing<I, F>
where
    I: ExactSizeIterator,
    F: FnMut(usize) -> I::Item,
{
}

impl<I, F> FusedIterator for PadUsing<I, F>
where
    I: FusedIterator,
    F: FnMut(usize) -> I::Item,
{
}
