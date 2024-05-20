use alloc::vec::Vec;
use std::fmt;
use std::iter::FusedIterator;

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
    iter: Option<I>,     // Inner iterator. Forget once consumed after 'base' iterations.
    items: Vec<I::Item>, // Fill from iter. Clear once adaptor is exhausted. Final length is 'base'.
    indices: Vec<usize>, // Indices just yielded. Clear once adaptor is exhausted. Length is 'pow'.
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
        items: Vec::new(),
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
        match (*pow, iter, items.len()) {
            // Final stable state: underlying iterator and items forgotten.
            (_, None, 0) => None,

            // Degenerated 0th power iteration.
            (0, Some(_), _) => {
                self.iter = None; // Forget without even consuming.
                Some((indices, items))
            }

            (pow, Some(it), 0) => {
                // Check whether there is at least one element in the iterator.
                if let Some(first) = it.next() {
                    // Allocate buffer to hold items about to be yielded.
                    items.reserve_exact(it.size_hint().0);
                    items.push(first);
                    // Same for indices to be yielded.
                    indices.reserve_exact(pow);
                    for _ in 0..pow {
                        indices.push(0);
                    }
                    return Some((indices, items));
                }
                // Degenerated iteration over an empty set, yet with non-null power.
                self.iter = None;
                None
            }

            (pow, Some(it), base) => {
                // We are still unsure whether all items have been collected.
                // As a consequence, 'base' is still uncertain,
                // but then we know that indices haven't started wrapping around yet.
                if let Some(next) = it.next() {
                    items.push(next);
                    indices[pow - 1] += 1;
                    return Some((indices, items));
                }

                // All items have just been collected.
                self.iter = None;
                if base == 1 || pow == 1 {
                    // End of iteration.
                    items.clear();
                    indices.clear();
                    return None;
                }

                // First wrap around.
                indices[pow - 1] = 0;
                indices[pow - 2] += 1;
                Some((indices, items))
            }

            (_, None, b) => {
                // Keep yielding items list, incrementing indices rightmost first.
                for index in indices.iter_mut().rev() {
                    *index += 1;
                    if *index < b {
                        return Some((indices, items));
                    }
                    *index = 0; // Wrap and increment left.
                }
                items.clear();
                indices.clear();
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

impl<I> FusedIterator for CartesianPower<I>
where
    I: Iterator,
    I::Item: Clone,
{
}

#[cfg(test)]
mod tests {
    //! Use chars and string to ease testing of every yielded iterator values.

    use super::CartesianPower;
    use crate::Itertools;
    use core::str::Chars;

    fn check_fused(mut exhausted_it: CartesianPower<Chars>, context: String) {
        for i in 0..100 {
            let act = exhausted_it.next();
            assert!(
                act.is_none(),
                "Iteration {} after expected exhaustion of {} \
                yielded {:?} instead of None. ",
                i,
                context,
                act,
            );
        }
    }

    #[test]
    fn basic() {
        fn check(origin: &str, pow: usize, expected: &[&str]) {
            let mut it = origin.chars().cartesian_power(pow);
            let mut i = 0;
            for exp in expected {
                let act = it.next();
                if act != Some(exp.chars().collect()) {
                    panic!(
                        "Failed iteration {} for {:?}^{}. \
                         Expected {:?}, got {:?} instead.",
                        i, origin, pow, exp, act,
                    );
                }
                i += 1;
            }
            check_fused(it, format!("iteration {} or {:?}^{}", i, origin, pow));
        }

        // Empty underlying iterator.
        check("", 0, &[""]);
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
            for &(n, exp) in expected {
                let act = it.nth(n);
                total_n.push(n);
                if act != exp.map(|s| s.chars().collect::<Vec<_>>()) {
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
                        act,
                    );
                }
            }
            check_fused(
                it,
                format!(
                    "nth({}) iteration of {:?}^{}",
                    total_n
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", "),
                    origin,
                    pow
                ),
            );
        }

        // HERE: make it work with the new implementation.

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
