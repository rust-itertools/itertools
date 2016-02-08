use super::misc::ToFloat;
use std::ops::{Add, Sub, Div, Mul};

/// An iterator of a sequence of evenly spaced floats.
///
/// Iterator element type is `F`.
#[derive(Clone, Debug)]
pub struct Linspace<F> {
    start: F,
    step: F,
    index: usize,
    len: usize,
}

impl<F> Iterator for Linspace<F>
    where F: Copy + Add<Output=F> + Mul<Output=F>,
          usize: ToFloat<F>,
{
    type Item = F;

    #[inline]
    fn next(&mut self) -> Option<F> {
        if self.index >= self.len {
            None
        } else {
            // Calculate the value just like numpy.linspace does
            let i = self.index;
            self.index += 1;
            Some(self.start + self.step * i.to_float())
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.len - self.index;
        (n, Some(n))
    }
}

impl<F> DoubleEndedIterator for Linspace<F>
    where F: Copy + Add<Output=F> + Mul<Output=F>,
          usize: ToFloat<F>,
{
    #[inline]
    fn next_back(&mut self) -> Option<F> {
        if self.index >= self.len {
            None
        } else {
            // Calculate the value just like numpy.linspace does
            self.len -= 1;
            let i = self.len;
            Some(self.start + self.step * i.to_float())
        }
    }
}

impl<F> ExactSizeIterator for Linspace<F> where Linspace<F>: Iterator { }

/// Return an iterator of evenly spaced floats.
///
/// The `Linspace` has `n` elements, where the first
/// element is `a` and the last element is `b`.
///
/// Iterator element type is `F`, where `F` must be
/// either `f32` or `f64`.
///
/// ```
/// use itertools::linspace;
///
/// itertools::assert_equal(linspace::<f32>(0., 1., 5),
///                         vec![0., 0.25, 0.5, 0.75, 1.0]);
/// ```
#[inline]
pub fn linspace<F>(a: F, b: F, n: usize) -> Linspace<F> where
    F: Copy + Sub<Output=F> + Div<Output=F> + Mul<Output=F>,
    usize: ToFloat<F>,
{
    let step = if n > 1 {
        let nf: F = n.to_float();
        (b - a)/(nf - 1.to_float())
    } else {
        0.to_float()
    };
    Linspace {
        start: a,
        step: step,
        index: 0,
        len: n,
    }
}
