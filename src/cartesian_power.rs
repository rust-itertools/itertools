use alloc::vec::Vec;
use std::fmt;

/// Pseudo-iterator owned by [`CartesianPower`],
/// yielding underlying indices by references to itself,
/// the [streaming iterator](https://docs.rs/streaming-iterator/latest/streaming_iterator/) way.
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
    pub fn new(base: usize, pow: u32) -> Self {
        Self {
            base,
            pow,
            values: None,
        }
    }
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

    fn size_hint(&self) -> (usize, Option<usize>) {
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

/// An adaptor iterating through all the ordered `n`-length lists of items
/// yielded by the underlying iterator, including repetitions.
/// Works by cloning the items into a fresh Vector on every `.next()`.
/// If this is too much allocation for you,
/// consider using the underlying lending [`Indices`] iterator instead.
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

/// Create a new `CartesianPower` from an iterator of clonables.
pub fn cartesian_power<I>(iter: I, pow: u32) -> CartesianPower<I>
where
    I: Iterator,
    I::Item: Clone,
{
    CartesianPower {
        iter: Some(iter),
        items: None,
        indices: Indices::new(0, pow),
    }
}

impl<I> CartesianPower<I>
where
    I: Iterator,
    I::Item: Clone,
{
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

/// Use chars and string to ease testing of every yielded iterator values.
#[cfg(test)]
mod tests {

    use crate::{cartesian_power::Indices, Itertools};

    #[test]
    fn next() {
        fn check(origin: &str, pow: u32, expected: &[&str]) {
            println!("================== ({origin:?}^{pow})");
            let mut it_act = origin
                .chars()
                .collect::<Vec<_>>() // Collect to get exact size hint upper bound.
                .into_iter()
                .cartesian_power(pow);
            // Check size_hint on the fly.
            let e_hint = expected.len();
            // Check thrice that it's cycling.
            for r in 1..=3 {
                println!("- - {r} - - - - - -");
                let mut it_exp = expected.iter();
                let mut i = 0;
                let mut e_remaining = e_hint;
                loop {
                    // Common context to emit in case of test failure.
                    let ctx = || {
                        format!(
                            "Failed iteration {} (repetition {}) for {:?}^{}.",
                            i, r, origin, pow,
                        )
                    };
                    // Check size hints.
                    let a_remaining = it_act.size_hint();
                    assert!(
                        if let (la, Some(ha)) = a_remaining {
                            la == e_remaining && ha == e_remaining
                        } else {
                            false
                        },
                        "{} Expected size hint: ({e}, Some({e})), got instead: {a:?}.",
                        ctx(),
                        e = e_remaining,
                        a = a_remaining,
                    );
                    // Actual/expected iterators steps.
                    let act = it_act.next();
                    let exp = it_exp.next().map(|e| e.chars().collect::<Vec<_>>());
                    println!(" {:?}", act);
                    // Possible failure report.
                    let fail = |e, a| {
                        let f = |o| {
                            if let Some(v) = o {
                                format!("{v:?}")
                            } else {
                                "None".into()
                            }
                        };
                        panic!("{} Expected {:?}, got {:?} instead.", ctx(), f(e), f(a));
                    };
                    // Comparison.
                    match (exp, act) {
                        (Some(exp), Some(act)) => {
                            if act != exp {
                                fail(Some(exp), Some(act));
                            }
                            i += 1;
                        }
                        (None, Some(act)) => {
                            fail(None, Some(act));
                        }
                        (Some(exp), None) => {
                            fail(Some(exp), None);
                        }
                        (None, None) => break,
                    }
                    e_remaining -= 1;
                }
            }
        }

        // Empty underlying iterator.
        check("", 0, &[""]); // 0^0 = 1.
        check("", 1, &[]);
        check("", 2, &[]);
        check("", 3, &[]);

        // Singleton underlying iterator.
        check("a", 0, &[""]);
        check("a", 1, &["a"]);
        check("a", 2, &["aa"]);
        check("a", 3, &["aaa"]);

        // Underlying pair.
        check("ab", 0, &[""]);
        check("ab", 1, &["a", "b"]);
        check("ab", 2, &["aa", "ab", "ba", "bb"]);
        check(
            "ab",
            3,
            &["aaa", "aab", "aba", "abb", "baa", "bab", "bba", "bbb"],
        );

        // Underlying triplet.
        check("abc", 0, &[""]);
        check("abc", 1, &["a", "b", "c"]);
        check(
            "abc",
            2,
            &["aa", "ab", "ac", "ba", "bb", "bc", "ca", "cb", "cc"],
        );
        check(
            "abc",
            3,
            &[
                "aaa", "aab", "aac", "aba", "abb", "abc", "aca", "acb", "acc", "baa", "bab", "bac",
                "bba", "bbb", "bbc", "bca", "bcb", "bcc", "caa", "cab", "cac", "cba", "cbb", "cbc",
                "cca", "ccb", "ccc",
            ],
        );
    }

    #[test]
    fn nth() {
        fn check(origin: &str, pow: u32, expected: &[(usize, (Option<&str>, usize))]) {
            println!("================== ({origin:?}^{pow})");
            let mut it = origin
                .chars()
                .collect::<Vec<_>>()
                .into_iter()
                .cartesian_power(pow);
            let mut total_n = Vec::new();
            for &(n, (exp, e_hint)) in expected {
                total_n.push(n);
                let ctx = || {
                    format!(
                        "Failed nth({}) iteration for {:?}^{}.",
                        total_n
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                            .join(", "),
                        origin,
                        pow,
                    )
                };
                let act = it.nth(n);
                let a_hint = it.size_hint();
                let act = act.map(|v| v.into_iter().collect::<String>());
                println!(
                    " → {}",
                    if let Some(act) = act.as_ref() {
                        act
                    } else {
                        "🗙"
                    }
                );
                if act.as_ref().map(String::as_str) != exp {
                    panic!("{} Expected {:?}, got {:?} instead.", ctx(), exp, act);
                }
                // Check size hint after stepping.
                assert!(
                    if let (la, Some(ha)) = a_hint {
                        la == e_hint && ha == e_hint
                    } else {
                        false
                    },
                    "{} Expected size hint: ({e}, Some({e})), got instead: {a:?}.",
                    ctx(),
                    e = e_hint,
                    a = a_hint,
                );
            }
        }

        // Ease test read/write.
        // Accept a sequence of '<n> <result>' yielded by a call to `.nth(n)`.
        macro_rules! check {
            ($base:expr, $pow:expr => $( $n:literal $expected:expr )+ ) => {
                check($base, $pow, &[$(($n, $expected)),+]);
            };
        }

        // Degenerated 0th power.
        let o = (None, 1);
        let e = (Some(""), 0); // "e"mpty.
        for base in ["", "a", "ab"] {
            check!(base, 0 => 0 e 0 o 0 e 0 o);
            check!(base, 0 => 0 e 1 o 0 e 1 o);
            check!(base, 0 => 0 e 2 o 1 o 0 e);
            check!(base, 0 => 1 o 0 e 0 o 1 o);
            check!(base, 0 => 1 o 1 o 0 e 0 o);
            check!(base, 0 => 1 o 2 o 0 e 1 o);
            check!(base, 0 => 2 o 0 e 1 o 0 e);
            check!(base, 0 => 2 o 1 o 2 o 0 e);
            check!(base, 0 => 2 o 2 o 0 e 2 o);
        }

        // Degenerated 0-base.
        let o = (None, 0);
        for pow in [1, 2, 3] {
            check!("", pow => 0 o 0 o 0 o 0 o);
            check!("", pow => 1 o 1 o 1 o 1 o);
            check!("", pow => 2 o 2 o 2 o 2 o);
            check!("", pow => 0 o 1 o 2 o 0 o);
            check!("", pow => 2 o 1 o 0 o 1 o);
        }

        // Unit power.
        let o = (None, 1);
        let a = (Some("a"), 0);
        check!("a", 1 => 0 a 0 o 0 a 0 o 0 a 0 o);
        check!("a", 1 => 1 o 1 o 1 o 1 o 1 o 1 o);
        check!("a", 1 => 2 o 2 o 2 o 2 o 2 o 2 o);
        check!("a", 1 => 0 a 1 o 0 a 1 o 0 a 1 o);
        check!("a", 1 => 1 o 0 a 1 o 0 a 1 o 0 a);
        check!("a", 1 => 0 a 2 o 0 a 2 o 0 a 2 o);
        check!("a", 1 => 2 o 0 a 2 o 0 a 2 o 0 a);
        check!("a", 1 => 1 o 2 o 1 o 2 o 1 o 2 o);
        check!("a", 1 => 2 o 1 o 2 o 1 o 2 o 1 o);
        check!("a", 1 => 0 a 1 o 2 o 0 a 1 o 2 o);
        check!("a", 1 => 0 a 2 o 1 o 0 a 2 o 1 o);
        check!("a", 1 => 1 o 0 a 2 o 1 o 0 a 2 o);
        check!("a", 1 => 1 o 2 o 0 a 1 o 2 o 0 a 1 o 2 o 0 a);
        check!("a", 1 => 2 o 0 a 1 o 2 o 0 a 1 o 2 o 0 a 1 o);
        check!("a", 1 => 2 o 1 o 0 a 2 o 1 o 0 a 2 o 1 o 0 a);
        check!("a", 1 => 1 o 0 a 0 o 1 o 0 a 0 o 1 o 0 a 0 o);
        check!("a", 1 => 1 o 1 o 0 a 0 o 1 o 1 o 0 a 0 o 1 o 1 o);

        let o = (None, 2);
        let a = (Some("a"), 1);
        let b = (Some("b"), 0);
        check!("ab", 1 => 0 a 0 b 0 o 0 a 0 b 0 o);
        check!("ab", 1 => 1 b 1 o 1 b 1 o 1 b 1 o);
        check!("ab", 1 => 2 o 2 o 2 o 2 o 2 o 2 o);
        check!("ab", 1 => 0 a 1 o 0 a 1 o 0 a 1 o);
        check!("ab", 1 => 1 b 0 o 1 b 0 o 1 b 0 o);
        check!("ab", 1 => 0 a 2 o 0 a 2 o 0 a 2 o);
        check!("ab", 1 => 2 o 0 a 2 o 0 a 2 o 0 a);
        check!("ab", 1 => 1 b 2 o 1 b 2 o 1 b 2 o);
        check!("ab", 1 => 2 o 1 b 2 o 1 b 2 o 1 b);
        check!("ab", 1 => 0 a 1 o 2 o 0 a 1 o 2 o);
        check!("ab", 1 => 0 a 2 o 1 b 0 o 2 o 1 b);
        check!("ab", 1 => 1 b 0 o 2 o 1 b 0 o 2 o);
        check!("ab", 1 => 1 b 2 o 0 a 1 o 2 o 0 a 1 o 2 o 0 a);
        check!("ab", 1 => 2 o 0 a 1 o 2 o 0 a 1 o);
        check!("ab", 1 => 2 o 1 b 0 o 2 o 1 b 0 o);
        check!("ab", 1 => 1 b 0 o 0 a 1 o 0 a 0 b 1 o 0 a 0 b);
        check!("ab", 1 => 1 b 1 o 0 a 0 b 1 o 1 b 0 o 0 a 1 o 1 b);

        let o = (None, 3);
        let a = (Some("a"), 2);
        let b = (Some("b"), 1);
        let c = (Some("c"), 0);
        check!("abc", 1 => 0 a 0 b 0 c 0 o 0 a 0 b 0 c 0 o);
        check!("abc", 1 => 1 b 1 o 1 b 1 o 1 b 1 o);
        check!("abc", 1 => 2 c 2 o 2 c 2 o 2 c 2 o);
        check!("abc", 1 => 0 a 1 c 0 o 1 b 0 c 1 o 0 a 1 c);
        check!("abc", 1 => 1 b 0 c 1 o 0 a 1 c 0 o 1 b 0 c);
        check!("abc", 1 => 0 a 2 o 0 a 2 o 0 a 2 o);
        check!("abc", 1 => 2 c 0 o 2 c 0 o 2 c 0 o);
        check!("abc", 1 => 1 b 2 o 1 b 2 o 1 b 2 o);
        check!("abc", 1 => 2 c 1 o 2 c 1 o 2 c 1 o);
        check!("abc", 1 => 0 a 1 c 2 o 0 a 1 c 2 o);
        check!("abc", 1 => 0 a 2 o 1 b 0 c 2 o 1 b);
        check!("abc", 1 => 1 b 0 c 2 o 1 b 0 c 2 o);
        check!("abc", 1 => 1 b 2 o 0 a 1 c 2 o 0 a 1 c 2 o 0 a);
        check!("abc", 1 => 2 c 0 o 1 b 2 o 0 a 1 c 2 o 0 a 1 c);
        check!("abc", 1 => 2 c 1 o 0 a 2 o 1 b 0 c 2 o 1 b 0 c);
        check!("abc", 1 => 1 b 0 c 0 o 1 b 0 c 0 o 1 b 0 c 0 o);
        check!("abc", 1 => 1 b 1 o 0 a 0 b 1 o 1 b 0 c 0 o 1 b 1 o);

        // Higher power.
        let o = (None, 1);
        let aa = (Some("aa"), 0);
        check!("a", 2 => 0 aa 0 o 0 aa 0 o 0 aa 0 o);
        check!("a", 2 => 1 o 1 o 1 o 1 o 1 o 1 o);
        check!("a", 2 => 2 o 2 o 2 o 2 o 2 o 2 o);
        check!("a", 2 => 0 aa 1 o 0 aa 1 o 0 aa 1 o);
        check!("a", 2 => 1 o 0 aa 1 o 0 aa 1 o 0 aa 1 o);
        check!("a", 2 => 0 aa 2 o 0 aa 2 o 0 aa 2 o);
        check!("a", 2 => 2 o 0 aa 2 o 0 aa 2 o 0 aa);
        check!("a", 2 => 1 o 2 o 1 o 2 o 1 o 2 o);
        check!("a", 2 => 2 o 1 o 2 o 1 o 2 o 1 o);
        check!("a", 2 => 0 aa 1 o 2 o 0 aa 1 o 2 o);
        check!("a", 2 => 0 aa 2 o 1 o 0 aa 2 o 1 o);
        check!("a", 2 => 1 o 0 aa 2 o 1 o 0 aa 2 o);
        check!("a", 2 => 1 o 2 o 0 aa 1 o 2 o 0 aa 1 o 2 o 0 aa);
        check!("a", 2 => 2 o 0 aa 1 o 2 o 0 aa 1 o 2 o 0 aa 1 o);
        check!("a", 2 => 2 o 1 o 0 aa 2 o 1 o 0 aa 2 o 1 o 0 aa);
        check!("a", 2 => 1 o 0 aa 0 o 1 o 0 aa 0 o 1 o 0 aa 0 o);
        check!("a", 2 => 1 o 1 o 0 aa 0 o 1 o 1 o 0 aa 0 o 1 o 1 o);

        let o = (None, 4);
        let aa = (Some("aa"), 3);
        let ab = (Some("ab"), 2);
        let ba = (Some("ba"), 1);
        let bb = (Some("bb"), 0);
        check!("ab", 2 => 0 aa 0 ab 0 ba 0 bb 0 o 0 aa 0 ab);
        check!("ab", 2 => 1 ab 1 bb 1 o 1 ab 1 bb 1 o);
        check!("ab", 2 => 2 ba 2 o 2 ba 2 o 2 ba 2 o);
        check!("ab", 2 => 0 aa 1 ba 0 bb 1 o 0 aa 1 ba);
        check!("ab", 2 => 1 ab 0 ba 1 o 0 aa 1 ba 0 bb 1 o 0 aa 1 ba 0 bb);
        check!("ab", 2 => 0 aa 2 bb 0 o 2 ba 0 bb 2 o 0 aa 2 bb);
        check!("ab", 2 => 2 ba 0 bb 2 o 0 aa 2 bb 0 o 2 ba 0 bb);
        check!("ab", 2 => 1 ab 2 o 1 ab 2 o 1 ab 2 o);
        check!("ab", 2 => 2 ba 1 o 2 ba 1 o 2 ba 1 o);
        check!("ab", 2 => 0 aa 1 ba 2 o 0 aa 1 ba 2 o);
        check!("ab", 2 => 0 aa 2 bb 1 o 0 aa 2 bb 1 o);
        check!("ab", 2 => 1 ab 0 ba 2 o 1 ab 0 ba 2 o);
        check!("ab", 2 => 1 ab 2 o 0 aa 1 ba 2 o 0 aa 1 ba 2 o 0 aa);
        check!("ab", 2 => 2 ba 0 bb 1 o 2 ba 0 bb 1 o 2 ba 0 bb 1 o);
        check!("ab", 2 => 2 ba 1 o 0 aa 2 bb 1 o 0 aa 2 bb 1 o 0 aa);
        check!("ab", 2 => 1 ab 0 ba 0 bb 1 o 0 aa 0 ab 1 bb 0 o 0 aa 1 ba 0 bb 0 o 1 ab 0 ba 0 bb);
        check!("ab", 2 => 1 ab 1 bb 0 o 0 aa 1 ba 1 o 0 aa 0 ab 1 bb 1 o 0 aa 0 ab 1 bb 1 o);

        let o = (None, 9);
        let aa = (Some("aa"), 8);
        let ab = (Some("ab"), 7);
        let ac = (Some("ac"), 6);
        let ba = (Some("ba"), 5);
        let bb = (Some("bb"), 4);
        let bc = (Some("bc"), 3);
        let ca = (Some("ca"), 2);
        let cb = (Some("cb"), 1);
        let cc = (Some("cc"), 0);
        check!("abc", 2 => 0 aa 0 ab 0 ac 0 ba 0 bb 0 bc 0 ca 0 cb 0 cc 0 o 0 aa 0 ab 0 ac 0 ba);
        check!("abc", 2 => 1 ab 1 ba 1 bc 1 cb 1 o 1 ab 1 ba 1 bc 1 cb 1 o 1 ab 1 ba 1 bc 1 cb 1 o);
        check!("abc", 2 => 2 ac 2 bc 2 cc 2 o 2 ac 2 bc 2 cc 2 o 2 ac 2 bc 2 cc 2 o 2 ac 2 bc 2 cc);
        check!("abc", 2 => 0 aa 1 ac 0 ba 1 bc 0 ca 1 cc 0 o 1 ab 0 ac 1 bb 0 bc 1 cb 0 cc 1 o);
        check!("abc", 2 => 1 ab 0 ac 1 bb 0 bc 1 cb 0 cc 1 o 0 aa 1 ac 0 ba 1 bc 0 ca 1 cc 0 o);
        check!("abc", 2 => 0 aa 2 ba 0 bb 2 cb 0 cc 2 o 0 aa 2 ba 0 bb 2 cb 0 cc 2 o 0 aa 2 ba);
        check!("abc", 2 => 2 ac 0 ba 2 ca 0 cb 2 o 0 aa 2 ba 0 bb 2 cb 0 cc 2 o 0 aa 2 ba 0 bb);
        check!("abc", 2 => 1 ab 2 bb 1 ca 2 o 1 ab 2 bb 1 ca 2 o 1 ab 2 bb 1 ca 2 o 1 ab 2 bb 1 ca);
        check!("abc", 2 => 2 ac 1 bb 2 cb 1 o 2 ac 1 bb 2 cb 1 o 2 ac 1 bb 2 cb 1 o 2 ac 1 bb 2 cb);
        check!("abc", 2 => 0 aa 1 ac 2 bc 0 ca 1 cc 2 o 0 aa 1 ac 2 bc 0 ca 1 cc 2 o 0 aa 1 ac);
        check!("abc", 2 => 0 aa 2 ba 1 bc 0 ca 2 o 1 ab 0 ac 2 bc 1 cb 0 cc 2 o 1 ab 0 ac 2 bc);
        check!("abc", 2 => 1 ab 0 ac 2 bc 1 cb 0 cc 2 o 1 ab 0 ac 2 bc 1 cb 0 cc 2 o 1 ab 0 ac);
        check!("abc", 2 => 1 ab 2 bb 0 bc 1 cb 2 o 0 aa 1 ac 2 bc 0 ca 1 cc 2 o 0 aa 1 ac 2 bc);
        check!("abc", 2 => 2 ac 0 ba 1 bc 2 cc 0 o 1 ab 2 bb 0 bc 1 cb 2 o 0 aa 1 ac 2 bc 0 ca);
        check!("abc", 2 => 2 ac 1 bb 0 bc 2 cc 1 o 0 aa 2 ba 1 bc 0 ca 2 o 1 ab 0 ac 2 bc 1 cb);
        check!("abc", 2 => 1 ab 0 ac 0 ba 1 bc 0 ca 0 cb 1 o 0 aa 0 ab 1 ba 0 bb 0 bc 1 cb 0 cc);
        check!("abc", 2 => 1 ab 1 ba 0 bb 0 bc 1 cb 1 o 0 aa 0 ab 1 ba 1 bc 0 ca 0 cb 1 o 1 ab);

        let o = (None, 27);
        let aaa = (Some("aaa"), 26);
        let aab = (Some("aab"), 25);
        let aac = (Some("aac"), 24);
        let aba = (Some("aba"), 23);
        let abb = (Some("abb"), 22);
        let abc = (Some("abc"), 21);
        let aca = (Some("aca"), 20);
        let acb = (Some("acb"), 19);
        let acc = (Some("acc"), 18);
        let baa = (Some("baa"), 17);
        let bab = (Some("bab"), 16);
        let bac = (Some("bac"), 15);
        let bba = (Some("bba"), 14);
        let bbb = (Some("bbb"), 13);
        let bbc = (Some("bbc"), 12);
        let bca = (Some("bca"), 11);
        let bcb = (Some("bcb"), 10);
        let bcc = (Some("bcc"), 9);
        let caa = (Some("caa"), 8);
        let cab = (Some("cab"), 7);
        let cac = (Some("cac"), 6);
        let cba = (Some("cba"), 5);
        let cbb = (Some("cbb"), 4);
        let cbc = (Some("cbc"), 3);
        let cca = (Some("cca"), 2);
        let ccb = (Some("ccb"), 1);
        let ccc = (Some("ccc"), 0);

        check!(
            "abc", 3 =>
            0 aaa
            0 aab
            0 aac
            0 aba
            0 abb
            0 abc
            0 aca
            0 acb
            0 acc
            0 baa
            0 bab
            0 bac
            0 bba
            0 bbb
            0 bbc
            0 bca
            0 bcb
            0 bcc
            0 caa
            0 cab
            0 cac
            0 cba
            0 cbb
            0 cbc
            0 cca
            0 ccb
            0 ccc
            0 o
            0 aaa
            0 aab
            0 aac
        );

        check!(
            "abc", 3 =>
            1 aab
            1 aba
            1 abc
            1 acb
            1 baa
            1 bac
            1 bbb
            1 bca
            1 bcc
            1 cab
            1 cba
            1 cbc
            1 ccb
            1 o
            1 aab
            1 aba
        );

        check!(
            "abc", 3 =>
            2 aac
            2 abc
            2 acc
            2 bac
            2 bbc
            2 bcc
            2 cac
            2 cbc
            2 ccc
            2 o
            2 aac
            2 abc
        );

        check!(
            "abc", 3 =>
            3 aba 3 acb 3 bac 3 bca 3 cab 3 cbc 3 o
            3 aba 3 acb 3 bac 3 bca 3 cab 3 cbc 3 o
        );

        check!(
            "abc", 3 =>
            4 abb 4 baa 4 bbc 4 cab 4 cca 4 o
            4 abb 4 baa 4 bbc 4 cab 4 cca 4 o
        );

        check!(
            "abc", 3 =>
            5 abc 5 bac 5 bcc 5 cbc 5 o
            5 abc 5 bac 5 bcc 5 cbc 5 o
        );

        check!("abc", 3 =>
            6 aca 6 bbb 6 cac 6 o
            6 aca 6 bbb 6 cac 6 o
        );

        check!("abc", 3 =>
            7 acb 7 bca 7 cbc 7 o
            7 acb 7 bca 7 cbc 7 o
        );

        check!(
            "abc", 3 =>
            8 acc 8 bcc 8 ccc 8 o
            8 acc 8 bcc 8 ccc 8 o
        );

        check!("abc", 3 => 9 baa 9 cab 9 o 9 baa 9 cab 9 o);
        check!("abc", 3 => 10 bab 10 cba 10 o 10 bab 10 cba 10 o);
        check!("abc", 3 => 11 bac 11 cbc 11 o 11 bac 11 cbc 11 o);
        check!("abc", 3 => 12 bba 12 ccb 12 o 12 bba 12 ccb 12 o);
        check!("abc", 3 => 13 bbb 13 o 13 bbb 13 o);
        check!("abc", 3 => 14 bbc 14 o 14 bbc 14 o);
        check!("abc", 3 => 25 ccb 25 o 25 ccb 25 o);
        check!("abc", 3 => 26 ccc 26 o 26 ccc 26 o);
        check!("abc", 3 => 27 o 27 o 27 o 27 o);
        check!("abc", 3 => 28 o 28 o 28 o 28 o);
    }
}
