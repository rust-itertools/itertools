use alloc::vec::Vec;
use std::fmt;

/// An adaptor iterating through all the ordered `n`-length lists of items
/// yielded by the underlying iterator, including repetitions.
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
    pow: usize,
    iter: Option<I>, // Inner iterator. Forget once consumed after 'base' iterations.
    items: Option<Vec<I::Item>>, // Fill from iter. Final length is 'base'.
    // None means that collection has not started yet.
    // Some(empty) means that collection would have started but pow = 0.

    // Indices just yielded. Length is 'pow'.
    // 0 0 .. 0 0 means that the first combination has been yielded.
    // 0 0 .. 0 1 means that the second combination has been yielded.
    // m m .. m m means that the last combination has just been yielded (m = base - 1).
    // b 0 .. 0 0 means that 'None' has just been yielded (b = base).
    // The latter is a special value marking the renewal of the iterator,
    // which can cycle again through another full round, ad libitum.
    indices: Vec<usize>,
}

/// Create a new `CartesianPower` from an iterator of clonables.
pub fn cartesian_power<I>(iter: I, pow: usize) -> CartesianPower<I>
where
    I: Iterator,
    I::Item: Clone,
{
    CartesianPower {
        pow,
        iter: Some(iter),
        items: None,
        indices: Vec::new(),
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
            pow,
            iter,
            items,
            indices,
        } = self;
        println!(
            "^{pow}: {} {indices:?}\t\t{:?}",
            if iter.is_some() { 'S' } else { 'N' },
            items.as_ref().map(Vec::len)
        );

        // (weird 'items @' bindings circumvent NLL limitations, unneeded with polonius)
        match (*pow, iter, &mut *items) {
            // First iteration with degenerated 0th power.
            (0, Some(_), items @ None) => {
                self.iter = None; // Forget about underlying iteration immediately.
                let empty = items.insert(Vec::new()); // Raise this value as a boolean flag.
                Some((indices, empty)) // Yield empty list.
            }

            // Subsequent degenerated 0th power iteration.
            // Use the Some<(empty)Vec> as a flag to alternate between yielding [] or None.
            (0, None, items @ Some(_)) => {
                *items = None;
                None
            }
            (0, None, items @ None) => Some((indices, items.insert(Vec::new()))),

            // First iteration in the general case.
            (pow, Some(it), items @ None) => {
                // Check whether there is at least one element in the iterator.
                if let Some(first) = it.next() {
                    items // Collect it.
                        .insert(Vec::with_capacity(it.size_hint().0))
                        .push(first);
                    // Prepare indices to be yielded.
                    indices.reserve_exact(pow);
                    for _ in 0..pow {
                        indices.push(0);
                    }
                    Some((indices, items.as_ref().unwrap()))
                } else {
                    // Degenerated iteration over an empty set:
                    // 'base = 0', yet with non-null power.
                    self.iter = None;
                    None
                }
            }

            // Stable iteration in the degenerated case 'base = 0'.
            (_, None, None) => None,

            // Subsequent iteration in the general case.
            (pow, Some(it), Some(items)) => {
                // We are still unsure whether all items have been collected.
                // As a consequence, the exact value of 'base' is still uncertain,
                // but then we know that indices haven't started wrapping around yet.
                if let Some(next) = it.next() {
                    items.push(next);
                    indices[pow - 1] += 1;
                    return Some((indices, items));
                }

                // The item collected on previous call was the last one.
                self.iter = None;
                let base = items.len(); // Definitive 'base' value.
                if base == 1 || pow == 1 {
                    // Early end of singleton iteration.
                    indices[0] = base; // Mark to cycle again on next iteration.
                    return None;
                }

                // First wrap around.
                indices[pow - 1] = 0;
                indices[pow - 2] = 1;
                Some((indices, items))
            }

            // Subsequent iteration in the general case after all items have been collected.
            (_, None, Some(items)) => {
                let base = items.len();
                if indices[0] == base {
                    // Special marker that iteration can start over for a new round.
                    indices[0] = 0;
                    return Some((indices, items));
                }
                // Keep yielding items list, incrementing indices rightmost first.
                if Self::inbounds_increment(indices, base) {
                    return Some((indices, items));
                }
                // Iteration is over.
                // Mark a special index value to not fuse the iterator
                // and make it possible to cycle through all results again.
                indices[0] = base;
                None
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

    /// Same as [`increment_indices`], but does n increments at once.
    /// The iterator is cycling, but `.nth()` does not 'wrap'
    /// and 'saturates' to None instead.
    fn increment_indices_by_n(&mut self, n: usize) -> Option<(&[usize], &[I::Item])> {
        let Self {
            pow,
            iter,
            items,
            indices,
        } = self;

        match (*pow, iter, &mut *items, n) {
            // First iteration with degenerated 0th power.
            (0, Some(_), items @ None, 0) => {
                // Same as .next().
                self.iter = None;
                let empty = items.insert(Vec::new());
                Some((indices, empty))
            }
            (0, Some(_), None, _) => {
                // Saturate.
                self.iter = None;
                None
            }

            // Subsequent degenerated 0th power iteration.
            // Same as `.next()`.
            (0, None, items @ None, 0) => {
                Some((indices, items.insert(Vec::new())))
            }
            // Saturate.
            (0, None, items, _) => {
                *items = None;
                None
            }

            // First iterations in the general case.
            // Possibly this will consume the entire underlying iterator,
            // but we need to consume to check.
            (pow, Some(it), items @ None, mut remaining) => {
                if let Some(first) = it.next() {
                    // There is at least one element in the iterator, prepare collection + indices.
                    let items = items.insert(Vec::with_capacity(it.size_hint().0));
                    items.push(first);
                    indices.reserve_exact(pow);
                    for _ in 0..pow {
                        indices.push(0);
                    }
                    // Collect more.
                    loop {
                        if remaining == 0 {
                            // Stop before collection completion.
                            indices[pow - 1] = n; // Hasn't wrapped yet.
                            return Some((indices, items));
                        }
                        if let Some(next) = it.next() {
                            items.push(next);
                            remaining -= 1;
                            continue;
                        }
                        // Collection completed, but we need to go further.
                        self.iter = None;
                        let base = items.len();
                        if Self::inbounds_increment_by(n, indices, base) {
                            return Some((indices, items));
                        }
                        // Immediate saturation.
                        indices[0] = base;
                        return None;
                    }
                } else {
                    // Degenerated iteration over an empty set.
                    self.iter = None;
                    None
                }
            }

            // Stable iteration in the degenerated case 'base = 0'.
            (_, None, None, _) => {
                None
            }

            // Subsequent iteration in the general case.
            // Again, immediate saturation is an option.
            (pow, Some(it), Some(items), mut remaining) => {
                if let Some(next) = it.next() {
                    items.push(next);
                    loop {
                        if remaining == 0 {
                            indices[pow - 1] += n + 1; // Hasn't wrapped yet.
                            return Some((indices, items));
                        }
                        if let Some(next) = it.next() {
                            items.push(next);
                            remaining -= 1;
                            continue;
                        }
                        break;
                    }
                }
                // Collection completed.
                self.iter = None;
                let base = items.len();
                if Self::inbounds_increment_by(n + 1, indices, base) {
                    return Some((indices, items));
                }
                // Saturate.
                indices[0] = base;
                None
            }

            // Subsequent iteration in the general case
            // after all items have been collected.
            (_, None, Some(items), n) => {
                let base = items.len();
                let shift = if indices[0] == base {
                    // Start over for a new round (already counted then).
                    indices[0] = 0;
                    0
                } else {
                    1
                };
                if Self::inbounds_increment_by(n + shift, indices, base) {
                    return Some((indices, items));
                }
                // Immediate re-saturation.
                indices[0] = base;
                None
            }
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
        todo!()
    }

    fn count(self) -> usize {
        todo!()
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
            pow,
            iter,
            items,
            indices,
        } = self;
        f.debug_struct("CartesianPower")
            .field("pow", pow)
            .field("iter", &iter.is_some())
            .field("items", items)
            .field("indices", indices)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    //! Use chars and string to ease testing of every yielded iterator values.

    use super::CartesianPower;
    use crate::Itertools;
    use core::str::Chars;

    #[test]
    fn basic() {
        fn check(origin: &str, pow: usize, expected: &[&str]) {
            println!("================== ({origin:?}^{pow})");
            let mut it_act = origin.chars().cartesian_power(pow);
            // Check thrice that it's cycling.
            for r in 1..=3 {
                println!("- - {r} - - - - - -");
                let mut it_exp = expected.iter();
                let mut i = 0;
                loop {
                    let act = it_act.next();
                    println!(" {:?}", act);
                    match (it_exp.next(), act) {
                        (Some(exp), Some(act)) => {
                            if act != exp.chars().collect::<Vec<_>>() {
                                panic!(
                                    "Failed iteration {} (repetition {}) for {:?}^{}. \
                                     Expected {:?}, got {:?} instead.",
                                    i, r, origin, pow, exp, act,
                                );
                            }
                            i += 1;
                        }
                        (None, Some(act)) => {
                            panic!(
                                "Failed iteration {} (repetition {}) for {:?}^{}. \
                                 Expected None, got {:?} instead.",
                                i, r, origin, pow, act,
                            );
                        }
                        (Some(exp), None) => {
                            panic!(
                                "Failed iteration {} (repetition {}) for {:?}^{}. \
                                 Expected {:?}, got None instead.",
                                i, r, origin, pow, exp,
                            );
                        }
                        (None, None) => break,
                    }
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
        fn check(origin: &str, pow: usize, expected: &[(usize, Option<&str>)]) {
            println!("================== ({origin:?}^{pow})");
            let mut it = origin.chars().cartesian_power(pow);
            let mut total_n = Vec::new();
            for &(n, exp) in expected {
                let act = it.nth(n);
                let act = act.map(|v| v.into_iter().collect::<String>());
                println!(
                    " → {}",
                    if let Some(act) = act.as_ref() {
                        act
                    } else {
                        "🗙"
                    }
                );
                total_n.push(n);
                if act.as_ref().map(String::as_str) != exp {
                    panic!(
                        "Failed nth({}) iteration for {:?}^{}. \
                             Expected {:?}, got {:?} instead.",
                        total_n
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                            .join(", "),
                        origin,
                        pow,
                        exp,
                        act
                    );
                }
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
        let o = None;
        let e = Some(""); // "e"mpty.

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
        for pow in [1, 2, 3] {
            check!("", pow => 0 o 0 o 0 o 0 o);
            check!("", pow => 1 o 1 o 1 o 1 o);
            check!("", pow => 2 o 2 o 2 o 2 o);
            check!("", pow => 0 o 1 o 2 o 0 o);
            check!("", pow => 2 o 1 o 0 o 1 o);
        }

        // Unit power.
        let a = Some("a");
        let b = Some("b");
        let c = Some("c");

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
        let aa = Some("aa");
        let ab = Some("ab");
        let ac = Some("ac");
        let ba = Some("ba");
        let bb = Some("bb");
        let bc = Some("bc");
        let ca = Some("ca");
        let cb = Some("cb");
        let cc = Some("cc");

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

        let s = |s| Some(s);

        check!(
            "abc", 3 =>
            0 s("aaa")
            0 s("aab")
            0 s("aac")
            0 s("aba")
            0 s("abb")
            0 s("abc")
            0 s("aca")
            0 s("acb")
            0 s("acc")
            0 s("baa")
            0 s("bab")
            0 s("bac")
            0 s("bba")
            0 s("bbb")
            0 s("bbc")
            0 s("bca")
            0 s("bcb")
            0 s("bcc")
            0 s("caa")
            0 s("cab")
            0 s("cac")
            0 s("cba")
            0 s("cbb")
            0 s("cbc")
            0 s("cca")
            0 s("ccb")
            0 s("ccc")
            0 o
            0 s("aaa")
            0 s("aab")
            0 s("aac")
        );

        check!(
            "abc", 3 =>
            1 s("aab")
            1 s("aba")
            1 s("abc")
            1 s("acb")
            1 s("baa")
            1 s("bac")
            1 s("bbb")
            1 s("bca")
            1 s("bcc")
            1 s("cab")
            1 s("cba")
            1 s("cbc")
            1 s("ccb")
            1 o
            1 s("aab")
            1 s("aba")
        );

        check!(
            "abc", 3 =>
            2 s("aac")
            2 s("abc")
            2 s("acc")
            2 s("bac")
            2 s("bbc")
            2 s("bcc")
            2 s("cac")
            2 s("cbc")
            2 s("ccc")
            2 o
            2 s("aac")
            2 s("abc")
        );

        check!(
            "abc", 3 =>
            3 s("aba") 3 s("acb") 3 s("bac") 3 s("bca") 3 s("cab") 3 s("cbc") 3 o
            3 s("aba") 3 s("acb") 3 s("bac") 3 s("bca") 3 s("cab") 3 s("cbc") 3 o
        );

        check!(
            "abc", 3 =>
            4 s("abb") 4 s("baa") 4 s("bbc") 4 s("cab") 4 s("cca") 4 o
            4 s("abb") 4 s("baa") 4 s("bbc") 4 s("cab") 4 s("cca") 4 o
        );

        check!(
            "abc", 3 =>
            5 s("abc") 5 s("bac") 5 s("bcc") 5 s("cbc") 5 o
            5 s("abc") 5 s("bac") 5 s("bcc") 5 s("cbc") 5 o
        );

        check!("abc", 3 =>
            6 s("aca") 6 s("bbb") 6 s("cac") 6 o
            6 s("aca") 6 s("bbb") 6 s("cac") 6 o
        );

        check!("abc", 3 =>
            7 s("acb") 7 s("bca") 7 s("cbc") 7 o
            7 s("acb") 7 s("bca") 7 s("cbc") 7 o
        );

        check!(
            "abc", 3 =>
            8 s("acc") 8 s("bcc") 8 s("ccc") 8 o
            8 s("acc") 8 s("bcc") 8 s("ccc") 8 o
        );

        check!("abc", 3 => 9 s("baa") 9 s("cab") 9 o 9 s("baa") 9 s("cab") 9 o);
        check!("abc", 3 => 10 s("bab") 10 s("cba") 10 o 10 s("bab") 10 s("cba") 10 o);
        check!("abc", 3 => 11 s("bac") 11 s("cbc") 11 o 11 s("bac") 11 s("cbc") 11 o);
        check!("abc", 3 => 12 s("bba") 12 s("ccb") 12 o 12 s("bba") 12 s("ccb") 12 o);
        check!("abc", 3 => 13 s("bbb") 13 o 13 s("bbb") 13 o);
        check!("abc", 3 => 14 s("bbc") 14 o 14 s("bbc") 14 o);
        check!("abc", 3 => 25 s("ccb") 25 o 25 s("ccb") 25 o);
        check!("abc", 3 => 26 s("ccc") 26 o 26 s("ccc") 26 o);
        check!("abc", 3 => 27 o 27 o 27 o 27 o);
        check!("abc", 3 => 28 o 28 o 28 o 28 o);
    }
}
