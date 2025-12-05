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
        match self.iter.next() {
            None => {
                if self.elements_from_next + self.elements_from_next_back < self.elements_required {
                    let e = Some((self.filler)(self.elements_from_next));
                    self.elements_from_next += 1;
                    e
                } else {
                    None
                }
            }
            e => {
                self.elements_from_next += 1;
                e
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (iter_lower, iter_upper) = self.iter.size_hint();
        let consumed = self
            .elements_from_next
            .saturating_add(self.elements_from_next_back);

        let total_lower = iter_lower
            .saturating_add(self.elements_from_next)
            .max(self.elements_required);
        let lower_bound = total_lower.saturating_sub(consumed);

        let upper_bound = iter_upper.map(|iter_upper| {
            let total_upper = iter_upper
                .saturating_add(self.elements_from_next)
                .max(self.elements_required);
            total_upper.saturating_sub(consumed)
        });

        (lower_bound, upper_bound)
    }

    fn fold<B, G>(self, mut init: B, mut f: G) -> B
    where
        G: FnMut(B, Self::Item) -> B,
    {
        let mut pos = self.elements_from_next;
        init = self.iter.fold(init, |acc, item| {
            pos += 1;
            f(acc, item)
        });
        (pos..self.elements_required).map(self.filler).fold(init, f)
    }
}

impl<I, F> DoubleEndedIterator for PadUsing<I, F>
where
    I: DoubleEndedIterator + ExactSizeIterator,
    F: FnMut(usize) -> I::Item,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let total_consumed = self.elements_from_next + self.elements_from_next_back;

        if self.iter.len() == 0 && total_consumed >= self.elements_required {
            return None;
        }

        let elements_remaining = self.elements_required.saturating_sub(total_consumed);
        self.elements_from_next_back += 1;

        if self.iter.len() < elements_remaining {
            Some((self.filler)(
                self.elements_required - self.elements_from_next_back,
            ))
        } else {
            let e = self.iter.next_back();
            assert!(e.is_some());
            e
        }
    }

    fn rfold<B, G>(self, mut init: B, mut f: G) -> B
    where
        G: FnMut(B, Self::Item) -> B,
    {
        let PadUsing {
            iter,
            elements_from_next,
            elements_from_next_back,
            mut elements_required,
            filler,
        } = self;
        let iter_len = iter.len();
        let original_iter_len = iter_len.saturating_add(elements_from_next);
        if elements_required < original_iter_len {
            elements_required = original_iter_len;
        }

        let start_idx = iter_len + elements_from_next;
        let end_idx = elements_required - elements_from_next_back;

        init = (start_idx..end_idx).rev().map(filler).fold(init, &mut f);

        iter.rfold(init, f)
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
