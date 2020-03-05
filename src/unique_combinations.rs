use std::fmt;

/// An iterator to iterate through all the `n`-length combinations in an iterator.
///
/// See [`.unique_combinations()`](../trait.Itertools.html#method.unique_combinations) for moref information.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct UniqueCombinations<I: Iterator> {
    indices: Vec<usize>,
    pool: Vec<I::Item>,
    first: bool,
    done: bool,
}

impl<I> fmt::Debug for UniqueCombinations<I>
where
    I: Iterator + fmt::Debug,
    I::Item: fmt::Debug,
{
    debug_fmt_fields!(Combinations, indices, pool, first, done);
}

/// Create a new `UniqueCombinations` from a iterator with clonable and sorable Items.
pub fn unique_combinations<I>(iter: I, len: usize) -> UniqueCombinations<I>
where
    I: Iterator,
    I::Item: Ord,
{
    let mut pool: Vec<_> = iter.collect();
    pool.sort_unstable();
    UniqueCombinations {
        indices: (0..len).collect(),
        pool,
        first: true,
        done: false, // only used on iterators with 0 length
    }
}

impl<I> UniqueCombinations<I>
where
    I: Iterator,
    I::Item: Ord + Clone,
{
    #[inline]
    fn generate(&self) -> Option<Vec<I::Item>> {
        Some(self.indices.iter().map(|n| self.pool[*n].clone()).collect())
    }
}

impl<I> Iterator for UniqueCombinations<I>
where
    I: Iterator,
    I::Item: Clone + Ord,
{
    type Item = Vec<I::Item>;
    fn next(&mut self) -> Option<Self::Item> {
        let len = self.indices.len();
        if self.first {
            // first pass throught
            if len == 0 {
                if self.done {
                    None
                } else {
                    self.done = true;
                    Some(Vec::new())
                }
            } else if len > self.pool.len() {
                None
            } else {
                self.first = false;
                self.generate()
            }
        } else {
            let org_len = self.pool.len();
            // check if we cant bump the back number
            if self.pool[self.indices[len - 1]] == self.pool[org_len - 1] {
                // locate the number closest behind that needs to be bumped
                for i in 2..len + 1 {
                    if self.pool[self.indices[len - i]] < self.pool[org_len - i] {
                        let lastpos = self.indices[len - i];
                        let val = &self.pool[lastpos];
                        for j in lastpos + 1..org_len {
                            if *val < self.pool[j] {
                                for k in 0..i {
                                    self.indices[len - i + k] = j + k;
                                }
                                return self.generate();
                            }
                        }
                    }
                }
                None
            } else {
                // bump the back number until value in pool increases
                let mut i = self.indices[len - 1];
                let current = &self.pool[i];
                let mut next = current;
                while current == next {
                    i += 1;
                    next = &self.pool[i];
                }
                self.indices[len - 1] = i;
                self.generate()
            }
        }
    }
}
