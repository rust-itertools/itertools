/// An iterator adaptor that pads a sequence to a minimum length by filling
/// missing elements using a closure.
///
/// Iterator element type is **I::Item**.
///
/// See [*.pad_tail_using()*](trait.Itertools.html#method.pad_tail_using) for more information.
#[derive(Clone, Debug)]
pub struct PadTailUsing<I, F> where
    I: Iterator,
    F: FnMut(usize) -> I::Item,
{
    iter: I,
    min: usize,
    pos: usize,
    filler: F,
}

impl<I, F> PadTailUsing<I, F> where
    I: Iterator,
    F: FnMut(usize) -> I::Item,
{
    /// Create a new `PadTailUsing` iterator.
    pub fn new(iter: I, min: usize, filler: F) -> PadTailUsing<I, F> {
        PadTailUsing {
            iter: iter,
            min: min,
            pos: 0,
            filler: filler,
        }
    }
}

impl<I, F> Iterator for PadTailUsing<I, F> where
    I: Iterator,
    F: FnMut(usize) -> I::Item,
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<I::Item> {
        match self.iter.next() {
            None => {
                if self.pos < self.min {
                    let e = Some((self.filler)(self.pos));
                    self.pos += 1;
                    e
                } else {
                    None
                }
            },
            e => {
                self.pos += 1;
                e
            }
        }
    }
}

impl<I, F> DoubleEndedIterator for PadTailUsing<I, F> where
    I: DoubleEndedIterator + ExactSizeIterator,
    F: FnMut(usize) -> I::Item,
{
    fn next_back(&mut self) -> Option<I::Item> {
        if self.min == 0 {
            self.iter.next_back()
        } else if self.iter.len() >= self.min {
            self.min -= 1;
            self.iter.next_back()
        } else {
            self.min -= 1;
            Some((self.filler)(self.min))
        }
    }
}
