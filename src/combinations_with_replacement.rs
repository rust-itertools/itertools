/// An iterator to iterate through all the `n`-length combinations in an iterator, with replacement.
///
/// See [`.combinations_with_replacement()`](../trait.Itertools.html#method.combinations_with_replacement) for more information.
#[derive(Debug, Clone)]
pub struct CombinationsWithReplacement<I: Iterator> {
    n: usize,
    indices: Vec<usize>,
    max_index: usize,
    pool: Vec<I::Item>,
    first: bool,
    empty: bool,
}

impl<I> CombinationsWithReplacement<I>
where
    I: Iterator,
    I::Item: Clone,
{
    /// Map the current mask over the pool to get an output combination
    fn current(&self) -> Vec<I::Item> {
        self.indices.iter().map(|i| self.pool[*i].clone()).collect()
    }
}

/// Create a new `CombinationsWithReplacement` from a clonable iterator.
pub fn combinations_with_replacement<I>(iter: I, n: usize) -> CombinationsWithReplacement<I>
where
    I: Iterator,
{
    let indices: Vec<usize> = vec![0; n];
    let pool: Vec<I::Item> = iter.collect();
    let empty = n == 0 || pool.len() == 0;
    let max_index = if empty { 0 } else { pool.len() - 1 };

    CombinationsWithReplacement {
        n,
        indices: indices,
        max_index,
        pool: pool,
        first: true,
        empty,
    }
}

impl<I> Iterator for CombinationsWithReplacement<I>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = Vec<I::Item>;
    fn next(&mut self) -> Option<Self::Item> {
        // If this is the first iteration
        if self.first {
            // In empty edge cases, stop iterating immediately
            return if self.empty {
                None
            // Otherwise, yield the initial state
            } else {
                self.first = false;
                Some(self.current())
            };
        }

        // Work out where we need to update our indices
        let mut increment: Option<(usize, usize)> = None;
        for (i, indices_int) in self.indices.iter().enumerate().rev() {
            if indices_int < &self.max_index {
                increment = Some((i, indices_int + 1));
                break;
            }
        }

        match increment {
            // If we can update the indices further
            Some((increment_from, increment_value)) => {
                // We need to update the rightmost non-max value
                // and all those to the right
                for indices_index in increment_from..self.indices.len() {
                    self.indices[indices_index] = increment_value
                }
                Some(self.current())
            }
            // Otherwise, we're done
            None => None,
        }
    }
}
