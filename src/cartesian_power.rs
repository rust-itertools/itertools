use alloc::vec::Vec;
use std::fmt;

/// Yield all indices combinations of size `pow` from `0..base`.
/// This "lending" iterator only allocates once,
/// and yields references to its internal owned slice of updated indices.
/// See [streaming iterator](https://docs.rs/streaming-iterator/latest/streaming_iterator/).
///
/// The resulting iterator is "cycling",
/// meaning that, after the last `.next()` call has yielded `None`,
/// you can call `.next()` again to resume iteration from the start,
/// as many times as needed.
///
/// This is the iterator internally used by [`CartesianPower`],
/// ```
/// use itertools::CartesianPowerIndices;
///
/// let mut it = CartesianPowerIndices::new(3, 2);
/// assert_eq!(it.next(), Some(&[0, 0][..]));
/// assert_eq!(it.next(), Some(&[0, 1][..]));
/// assert_eq!(it.next(), Some(&[0, 2][..]));
/// assert_eq!(it.next(), Some(&[1, 0][..]));
/// assert_eq!(it.next(), Some(&[1, 1][..]));
/// assert_eq!(it.next(), Some(&[1, 2][..]));
/// assert_eq!(it.next(), Some(&[2, 0][..]));
/// assert_eq!(it.next(), Some(&[2, 1][..]));
/// assert_eq!(it.next(), Some(&[2, 2][..]));
/// assert_eq!(it.next(), None);              // End of iteration.
/// assert_eq!(it.next(), Some(&[0, 0][..])); // Cycle: start over.
/// assert_eq!(it.next(), Some(&[0, 1][..]));
/// // ...
///
/// let mut it = CartesianPowerIndices::new(2, 3);
/// assert_eq!(it.next(), Some(&[0, 0, 0][..]));
/// assert_eq!(it.next(), Some(&[0, 0, 1][..]));
/// assert_eq!(it.next(), Some(&[0, 1, 0][..]));
/// assert_eq!(it.next(), Some(&[0, 1, 1][..]));
/// assert_eq!(it.next(), Some(&[1, 0, 0][..]));
/// assert_eq!(it.next(), Some(&[1, 0, 1][..]));
/// assert_eq!(it.next(), Some(&[1, 1, 0][..]));
/// assert_eq!(it.next(), Some(&[1, 1, 1][..]));
/// assert_eq!(it.next(), None);
/// assert_eq!(it.next(), Some(&[0, 0, 0][..]));
/// assert_eq!(it.next(), Some(&[0, 0, 1][..]));
/// // ...
/// ```
#[derive(Debug, Clone)]
pub struct Indices {
    // May be incremented by owner on first pass as long as exact value is unknown.
    base: usize,
    pow: u32,

    // Indices just yielded. Length is 'pow'.
    // 0 0 .. 0 0 means that the first combination has been yielded.
    // 0 0 .. 0 1 means that the second combination has been yielded.
    // m m .. m m means that the last combination has just been yielded (m = base - 1).
    // b 0 .. 0 0 means that 'None' has just been yielded (b = base).
    // The latter is a special value marking the renewal of the iterator,
    // which can cycle again through another full round, ad libitum.
    values: Option<Vec<usize>>,
}

impl Indices {
    /// Create a new `base^pow` lending iterator.
    pub fn new(base: usize, pow: u32) -> Self {
        Self {
            base,
            pow,
            values: None,
        }
    }

    /// Step the iterator, yielding a reference to internal updated indices,
    /// or `None` if the iteration is exhausted.
    #[allow(clippy::should_implement_trait)] // <- Intended `.next` name "like Iterator::next".
    pub fn next(&mut self) -> Option<&[usize]> {
        let Self { base, pow, values } = self;

        match (base, pow, values) {
            // First iteration with degenerated 0th power.
            (_, 0, values @ None) => Some(values.insert(Vec::new())),

            // Last degenerated 0th power iteration.
            // Use the Some<(empty)Vec> as a flag to alternate between yielding [] or None.
            (_, 0, values @ Some(_)) => {
                *values = None;
                None
            }

            // Stable iteration in 0-base.
            (0, _, _) => None,

            // First iteration in the general case.
            (_, pow, values @ None) => Some(values.insert(vec![0; *pow as usize])),

            // Subsequent iteration in the general case.
            (&mut base, _, Some(values)) => {
                if values[0] == base {
                    // Special marker that iteration can start over for a new round.
                    values[0] = 0;
                    return Some(values);
                }
                if inbounds_increment(values, base) {
                    return Some(values);
                }
                // Iteration is over.
                // Mark a special index value to not fuse the iterator
                // and make it possible to cycle through all results again.
                values[0] = base;
                None
            }
        }
    }

    /// Same as [`next`][crate::CartesianPowerIndices::next],
    /// but skip `n` steps.
    /// Return `None` if this would lead to iterator exhaustion.  
    /// Saturates in case of overflow:
    /// the iterator is cycling,
    /// but if you skip past the last iteration,
    /// you'll obtain `None` no matter how far you skip.
    /// Iteration will only resume on further calls to `.next()` and `.nth()`.
    ///
    /// ```
    /// use itertools::CartesianPowerIndices;
    ///
    /// let mut it = CartesianPowerIndices::new(3, 2);
    /// assert_eq!(it.nth(0), Some(&[0, 0][..]));
    /// assert_eq!(it.nth(1), Some(&[0, 2][..]));
    /// assert_eq!(it.nth(2), Some(&[1, 2][..]));
    /// assert_eq!(it.nth(9), None); // Overshoot, but don't resume cycling yet.
    /// assert_eq!(it.nth(2), Some(&[0, 2][..])); // Only resume cycling now.
    /// ```
    pub fn nth(&mut self, n: usize) -> Option<&[usize]> {
        let Self { base, pow, values } = self;
        match (base, pow, values, n) {
            // First iteration with degenerated 0th power.
            (_, 0, values @ None, 0) => {
                Some(values.insert(Vec::new())) // Same as .next()
            }
            // Saturate.
            (_, 0, values, _) => {
                *values = None;
                None
            }
            // Stable iteration in 0-base.
            (0, _, _, _) => None,
            // First iteration in the general case.
            (&mut base, pow, values @ None, n) => {
                let values = values.insert(vec![0; *pow as usize]);
                if inbounds_increment_by(n, values, base) {
                    return Some(values);
                }
                // Immediate saturation.
                values[0] = base;
                None
            }
            // Subsequent iteration in the general case.
            (&mut base, _, Some(values), n) => {
                let shift = if values[0] == base {
                    // Start over for a new round (already counted then).
                    values[0] = 0;
                    0
                } else {
                    1
                };
                if inbounds_increment_by(n + shift, values, base) {
                    return Some(values);
                }
                // Immediate re-saturation.
                values[0] = base;
                None
            }
        }
    }

    /// Akin to [`Iterator::size_hint`].
    pub fn size_hint(&self) -> (usize, Option<usize>) {
        self.size_hint_with_base(self.base)
    }

    fn size_hint_with_base(&self, base: usize) -> (usize, Option<usize>) {
        let Self {
            base: _,
            pow,
            values,
        } = self;

        // The following case analysis matches implementation of `.next()`.
        match (base, pow, values) {
            // First iteration with degenerated 0th power.
            (_, 0, None) => (1, Some(1)),

            // Last degenerated 0th power iteration | Stable iteration in 0-base.
            // Use the Some<(empty)Vec> as a flag to alternate between yielding [] or None.
            (0, _, _) | (_, 0, Some(_)) => (0, Some(0)),

            // First iteration in the general case.
            (base, &pow, None) => {
                let c = base.checked_pow(pow);
                (c.unwrap_or(usize::MAX), c)
            }

            // Subsequent iteration in the general case.
            (base, &pow, Some(values)) => {
                if values[0] == base {
                    // Fresh re-start.
                    let r = base.checked_pow(pow);
                    return (r.unwrap_or(usize::MAX), r);
                }
                // Count what's left from current indices.
                // This is subtracting the current iteration number base^pow,
                // using the complement method.
                let calc = || -> Option<usize> {
                    // (closure-wrap to ease interruption on overflow with ?-operator)
                    let mut r = 0usize;
                    for (&i, rank) in values.iter().rev().zip(0u32..) {
                        let complement = base - 1 - i;
                        let increment = complement.checked_mul(base.checked_pow(rank)?)?;
                        r = r.checked_add(increment)?;
                    }
                    Some(r)
                };
                let Some(r) = calc() else {
                    return (usize::MAX, None);
                };
                (r, Some(r))
            }
        }
    }
}

/// Increment indices, returning false in case of overflow.
fn inbounds_increment(indices: &mut [usize], base: usize) -> bool {
    for index in indices.iter_mut().rev() {
        *index += 1;
        if *index < base {
            return true;
        }
        *index = 0; // Wrap and increment left.
    }
    false
}

/// Increment indices by n, returning false in case of (saturating) overflow.
fn inbounds_increment_by(n: usize, indices: &mut [usize], base: usize) -> bool {
    let mut q = n;
    for index in indices.iter_mut().rev() {
        let s = *index + q;
        q = s / base;
        *index = s % base;
        if q == 0 {
            return true;
        }
    }
    // Saturation requires a second pass to reset all indices.
    for index in indices.iter_mut() {
        *index = 0;
    }
    false
}

/// The adaptor adaptor yielded by
/// [`.cartesian_power()`](crate::Itertools::cartesian_power).
///
/// This iterator is *cycling*,
/// meaning that, once consumed after `.next()` returns `None`,
/// you can call `.next()` again to resume iteration from the start.
///
/// See [`.cartesian_power()`](crate::Itertools::cartesian_power)
/// for more information.
#[derive(Clone)]
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct CartesianPower<I>
where
    I: Iterator,
    I::Item: Clone,
{
    iter: Option<I>, // Inner iterator. Forget once consumed after 'base' iterations.
    items: Option<Vec<I::Item>>, // Fill from 'iter'. Final length is 'base'.
    // Keep track of the items to yield,
    // updating 'base' as the inner iterator is consumed.
    indices: Indices,
}

impl<I> CartesianPower<I>
where
    I: Iterator,
    I::Item: Clone,
{
    pub(crate) fn new(iter: I, pow: u32) -> CartesianPower<I> {
        CartesianPower {
            iter: Some(iter),
            items: None,
            indices: Indices::new(0, pow),
        }
    }

    /// Increments internal indices to advance to the next list to be yielded.
    /// This collects new items from the underlying iterator
    /// if they were not all already collected.
    ///
    /// Returns None if we've run out of possible lists,
    /// otherwise return refs to the indices to yield next,
    /// valid within the collected items slice also returned.
    fn increment_indices(&mut self) -> Option<(&[usize], &[I::Item])> {
        let Self {
            iter,
            items,
            indices,
        } = self;

        if let Some(iter) = iter {
            let items = items.get_or_insert_with(|| Vec::with_capacity(iter.size_hint().0));
            if let Some(new) = iter.next() {
                indices.base += 1;
                items.push(new);
            } else {
                self.iter = None;
            }
            indices.next().map(move |i| (i, items.as_slice()))
        } else if let Some(items) = items {
            indices.next().map(move |i| (i, items.as_slice()))
        } else {
            indices.next().map(move |i| (i, [].as_slice()))
        }
    }

    /// Same as [`increment_indices`], but does n increments at once.
    /// The iterator is cycling, but `.nth()` does not 'wrap'
    /// and 'saturates' to None instead.
    fn increment_indices_by_n(&mut self, n: usize) -> Option<(&[usize], &[I::Item])> {
        let Self {
            iter,
            items,
            indices,
        } = self;

        if let Some(iter) = iter {
            let items = items.get_or_insert_with(|| Vec::with_capacity(iter.size_hint().0));
            for _ in 0..=n {
                if let Some(new) = iter.next() {
                    indices.base += 1;
                    items.push(new);
                } else {
                    self.iter = None;
                    break;
                }
            }
            indices.nth(n).map(move |i| (i, items.as_slice()))
        } else if let Some(items) = items {
            indices.nth(n).map(move |i| (i, items.as_slice()))
        } else {
            indices.nth(n).map(move |i| (i, [].as_slice()))
        }
    }
}

impl<I> Iterator for CartesianPower<I>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        // If anything to yield,
        // clone the correct 'pow' instances of collected items
        // into a freshly allocated vector.
        self.increment_indices().map(|(indices, items)| {
            indices
                .iter()
                .map(|&i| items[i].clone())
                .collect::<Vec<_>>()
        })
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.increment_indices_by_n(n).map(|(indices, items)| {
            indices
                .iter()
                .map(|&i| items[i].clone())
                .collect::<Vec<_>>()
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { iter, indices, .. } = self;
        if let Some(iter) = iter {
            let yet = indices.base; // <- The number of items yielded so far.
            let (a, b) = iter.size_hint(); // <- The estimated number of remaining items.
            let a = indices.size_hint_with_base(yet + a).0;
            let b = b.and_then(|b| indices.size_hint_with_base(yet + b).1);
            (a, b)
        } else {
            indices.size_hint()
        }
    }

    fn count(self) -> usize {
        self.size_hint().0
    }
}

// Elide underlying iterator from the debug display.
impl<I> fmt::Debug for CartesianPower<I>
where
    I: Iterator + fmt::Debug,
    I::Item: fmt::Debug + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            iter,
            items,
            indices,
        } = self;
        f.debug_struct("CartesianPower")
            .field("iter", &iter.is_some())
            .field("items", items)
            .field("indices", indices)
            .finish()
    }
}
