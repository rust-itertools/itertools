/// An iterator to iterate through all product combinations of an iterator.
///
/// See [`.product_combination()`](../trait.Itertools.html#method.product_combination) for more information.
pub struct ProductCombination<I: Iterator> {
    data: Vec<I::Item>,
    internal_state: Vec<usize>
}

/// Create a new `ProductionCombination` from a clonable iterator.
pub fn product_combination<I>(iter: I) -> ProductCombination<I>
    where I: Iterator
{
    //Clone all the elements
    let mut data: Vec<I::Item> = iter.collect();

    ProductCombination {
        data,
        internal_state: Vec::new(),
    }
}

impl<I: Iterator> ProductCombination<I> {
    //Product combinations are formed from a number which has radix data.len() where each digit is the index of an element.
    //Incrementing this base data.len() number will produce all possible product combinations.
    fn increment(&mut self) {
        let mut pointer: usize = 0;
        let mut tack_on = false;
        loop {
            match self.internal_state.get_mut(pointer) {
                None => {
                    tack_on = true;
                    break;
                },
                Some(n) => {
                    *n += 1;
                    *n %= self.data.len();

                    if *n == 0 {
                        pointer += 1;
                    } else {
                        break;
                    }
                },
            }
        }
        if tack_on {
            self.internal_state.push(0);
        }
    }
}

impl<I> Iterator for ProductCombination<I>
    where I: Iterator, I::Item: Clone
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        self.increment();

        //Map each digit from the base data.len() to its corresponding element
        let mut result: Self::Item = self.internal_state.iter().map(|idx| {
            return self.data[*idx].clone();
        }).collect();

        return Some(result);
    }
}