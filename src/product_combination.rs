/// An iterator to iterate through all product combinations of an iterator.
///
/// See [`.product_combination()`](../trait.Itertools.html#method.product_combination) for more information.
pub struct ProductCombination<I: Iterator + Clone> {
    data: I,
    internal_state: Vec<usize>
}

/// Create a new `ProductionCombination` from a clonable iterator.
pub fn product_combination<I>(iter: I) -> ProductCombination<I>
    where I: Iterator + Clone
{
    ProductCombination {
        data: iter,
        internal_state: Vec::new(),
    }
}

impl<I: Iterator + Clone> ProductCombination<I> {
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

                    match self.data.clone().nth(*n) {
                        Some(_) => {},
                        None => {
                            //No such item exists ... start back at the beginning
                            *n = 0;
                        }
                    }

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
    where I: Iterator + Clone
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        self.increment();

        //Map each digit from the base data.len() to its corresponding element
        Some(self.internal_state.iter().map(|idx| self.data.clone().nth(*idx).unwrap()).collect())
    }
}