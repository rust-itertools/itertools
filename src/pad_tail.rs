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
    min: usize,
    pos: usize,
    back: usize,
    total_len: usize,
    filler: F,
}

impl<I, F> std::fmt::Debug for PadUsing<I, F>
where
    I: std::fmt::Debug,
{
    debug_fmt_fields!(PadUsing, iter, min, pos, back, total_len);
}

/// Create a new `PadUsing` iterator.
pub fn pad_using<I, F>(iter: I, min: usize, filler: F) -> PadUsing<I, F>
where
    I: Iterator,
    F: FnMut(usize) -> I::Item,
{
    PadUsing {
        iter: iter.fuse(),
        min,
        pos: 0,
        back: 0,
        total_len: min,
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
                if self.pos + self.back < self.total_len {
                    let e = Some((self.filler)(self.pos));
                    self.pos += 1;
                    e
                } else {
                    None
                }
            }
            e => {
                self.pos += 1;
                e
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (iter_lower, iter_upper) = self.iter.size_hint();
        let consumed = self.pos.saturating_add(self.back);

        let total_lower = iter_lower.saturating_add(self.pos).max(self.min);
        let lower_bound = total_lower.saturating_sub(consumed);

        let upper_bound = iter_upper.map(|iter_upper| {
            let total_upper = iter_upper.saturating_add(self.pos).max(self.min);
            total_upper.saturating_sub(consumed)
        });

        (lower_bound, upper_bound)
    }

    fn fold<B, G>(self, mut init: B, mut f: G) -> B
    where
        G: FnMut(B, Self::Item) -> B,
    {
        let mut pos = self.pos;
        init = self.iter.fold(init, |acc, item| {
            pos += 1;
            f(acc, item)
        });
        (pos..self.min).map(self.filler).fold(init, f)
    }
}

impl<I, F> DoubleEndedIterator for PadUsing<I, F>
where
    I: DoubleEndedIterator + ExactSizeIterator,
    F: FnMut(usize) -> I::Item,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let current_iter_len = self.iter.len();
        let original_iter_len = current_iter_len.saturating_add(self.pos);
        if self.total_len < original_iter_len {
            self.total_len = original_iter_len;
        }

        if self.pos + self.back >= self.total_len {
            return None;
        }

        let padding_count = self.total_len.saturating_sub(current_iter_len + self.pos);

        if self.back < padding_count {
            let idx = self.total_len - self.back - 1;
            self.back += 1;
            Some((self.filler)(idx))
        } else {
            self.back += 1;
            self.iter.next_back()
        }
    }

    fn rfold<B, G>(self, mut init: B, mut f: G) -> B
    where
        G: FnMut(B, Self::Item) -> B,
    {
        let PadUsing {
            iter,
            min: _,
            pos,
            back,
            mut total_len,
            filler,
        } = self;
        let iter_len = iter.len();
        let original_iter_len = iter_len.saturating_add(pos);
        if total_len < original_iter_len {
            total_len = original_iter_len;
        }

        let start_idx = iter_len + pos;
        let end_idx = total_len - back;

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
