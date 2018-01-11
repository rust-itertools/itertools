pub fn replace<T, I, R>(iter: I, needle: T, replacement: R) -> Replace<T, I, R>
    where
        T: Eq,
        I: Iterator<Item=T>,
        R: Iterator<Item=T> + Clone,
{
    Replace { needle: needle, iter: iter, with_orig: replacement, with: None }
}

/// An iterator adaptor that replaces occurrences of an item with a different sequence of items.
///
/// It is returned by [`Itertools::replace`].
///
/// [`Itertools::replace`]: ../trait.Itertools.html#method.replace
pub struct Replace<T, I, R>
    where
        T: Eq,
        I: Iterator<Item=T>,
        R: Iterator<Item=T> + Clone
{
    needle: I::Item,
    iter: I,
    with: Option<R>,
    with_orig: R,
}

impl<T, I, R> Iterator for Replace<T, I, R>
    where
        T: Eq,
        I: Iterator<Item=T>,
        R: Iterator<Item=T> + Clone,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<I::Item> {
        loop {
            self.with = match self.with {
                Some(ref mut replacement) => {
                    match replacement.next() {
                        Some(item) => {
                            // emit replacement item
                            return Some(item);
                        }
                        None => {
                            // continue with original iterator
                            None
                        }
                    }
                }
                None => {
                    match self.iter.next() {
                        None => return None,
                        Some(item) => if item == self.needle {
                            // emit the replacement iterator once
                            Some(self.with_orig.clone())
                        } else {
                            return Some(item);
                        }
                    }
                }
            }
        }
    }
}
