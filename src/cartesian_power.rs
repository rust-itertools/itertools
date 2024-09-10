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
                for index in indices.iter_mut().rev() {
                    *index += 1;
                    if *index < base {
                        return Some((indices, items));
                    }
                    *index = 0; // Wrap and increment left.
                }
                // Iteration is over.
                // Mark a special index value to not fuse the iterator
                // and make it possible to cycle through all results again.
                indices[0] = base;
                None
            }
        }
    }

    /// Same as [`increment_indices`], but does n increments at once.
    fn increment_indices_by_n(&mut self, n: usize) -> Option<(&[usize], &[I::Item])> {
        todo!()
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
                    match (it_exp.next(), it_act.next()) {
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
            let mut it = origin.chars().cartesian_power(pow);
            let mut total_n = Vec::new();
            for r in 1..=3 {
                for &(n, exp) in expected {
                    let act = it.nth(n);
                    total_n.push(n);
                    if act != exp.map(|s| s.chars().collect::<Vec<_>>()) {
                        panic!(
                            "Failed nth({}) iteration (repetition {}) for {:?}^{}. \
                             Expected {:?}, got {:?} instead.",
                            total_n
                                .iter()
                                .map(ToString::to_string)
                                .collect::<Vec<_>>()
                                .join(", "),
                            r,
                            origin,
                            pow,
                            exp,
                            act
                        );
                    }
                }
            }
        }

        // Check degenerated cases.
        check("", 0, &[(0, Some("")), (0, None)]);
        check("", 0, &[(0, Some("")), (1, None)]);
        check("", 0, &[(0, Some("")), (2, None)]);
        check("", 0, &[(1, None), (0, None)]);
        check("", 0, &[(1, None), (1, None)]);
        check("", 0, &[(1, None), (2, None)]);
        check("", 0, &[(2, None), (0, None)]);
        check("", 0, &[(2, None), (1, None)]);
        check("", 0, &[(2, None), (2, None)]);

        check("a", 0, &[(0, Some("")), (0, None)]);
        check("a", 0, &[(0, Some("")), (1, None)]);
        check("a", 0, &[(0, Some("")), (2, None)]);
        check("a", 0, &[(1, None), (0, None)]);
        check("a", 0, &[(1, None), (1, None)]);
        check("a", 0, &[(1, None), (2, None)]);
        check("a", 0, &[(2, None), (0, None)]);
        check("a", 0, &[(2, None), (1, None)]);
        check("a", 0, &[(2, None), (2, None)]);

        // Unit power.
        check("a", 1, &[(0, Some("a")), (0, None)]);
        check("a", 1, &[(0, Some("a")), (1, None)]);
        check("a", 1, &[(0, Some("a")), (2, None)]);
        check("a", 1, &[(1, None), (0, None)]);
        check("a", 1, &[(1, None), (1, None)]);
        check("a", 1, &[(1, None), (2, None)]);
        check("a", 1, &[(2, None), (0, None)]);
        check("a", 1, &[(2, None), (1, None)]);
        check("a", 1, &[(2, None), (2, None)]);

        check("ab", 1, &[(0, Some("a")), (0, Some("b")), (0, None)]);
        check("ab", 1, &[(1, Some("b")), (0, None), (0, None)]);
        check("ab", 1, &[(2, None), (0, None), (0, None)]);
    }
}
