/// An iterator to iterate through all product combinations of an iterator.
///
/// See [`.product_combination()`](../trait.Itertools.html#method.product_combination) for more information.
pub struct ProductCombination<I: Iterator + Clone> {
    data: I,
    //Internal state is a vec of tuples containing an iterator and the last retrieved item from this iterator
    internal_state: Vec<(I, I::Item)>
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
    fn increment(&mut self) {
        let mut pointer: usize = 0;
        let mut tack_on = false;
        loop {
            match self.internal_state.get_mut(pointer) {
                None => {
                    tack_on = true;
                    break;
                },
                Some((ref mut iter, ref mut item)) => {
                    let new_item = iter.next();
                    match new_item {
                        //Reached the end of the iterator ... restart the iterator and advance the pointer
                        None => {
                            *iter = self.data.clone();
                            match iter.next() {
                                None => {
                                    panic!("0 sized iterator");
                                },
                                Some(i) => {
                                    *item = i;
                                }
                            }
                            pointer += 1;
                        },
                        Some(i) => {
                            *item = i;
                            break;
                        }
                    }
                },
            }
        }
        if tack_on {
            let mut new_iter = self.data.clone();
            match new_iter.next() {
                None => {
                    panic!("0 sized iterator");
                },
                Some(new_item) => {
                    self.internal_state.push((new_iter, new_item));
                }
            }
        }
    }
}

impl<I> Iterator for ProductCombination<I>
    where I: Iterator + Clone, I::Item: Clone
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        self.increment();

        Some(self.internal_state.iter().map(|(ref iter, ref item)| item.clone()).collect())
    }
}