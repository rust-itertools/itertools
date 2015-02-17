
use std::iter;
use std::num::{Float, NumCast};

/// An iterator of `n` evenly spaced floats.
///
/// Iterator element type is `F`.
pub type Linspace<F> = iter::Take<iter::Counter<F>>;

/// Return an iterator with `n` elements, where the first
/// element is `a` and the last element is `b`.
///
/// Iterator element type is `F`.
///
/// ```
/// use itertools as it;
/// let mut xs = it::linspace::<f32>(0., 1., 5);
/// assert_eq!(xs.collect::<Vec<_>>(),
///            vec![0., 0.25, 0.5, 0.75, 1.0]);
/// ```
#[inline]
pub fn linspace<F: Float>(a: F, b: F, n: usize) -> Linspace<F>
{
    let nf: F = NumCast::from(n).unwrap();
    let step = (b - a)/(nf - Float::one());
    iter::count(a, step).take(n)
}
