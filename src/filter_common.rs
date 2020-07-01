/// Filters items from the original iterator that exist in the filter list.
///
/// Iterator element type is `I::Item`.
///
/// See [`.filter_common()`](../trait.Itertools.html#method.filter_common) for more information.
pub struct FilterCommon<I, K>
    where
        I: Iterator,
        K: PartialEq<I::Item>,
{
    iterator: I,
    filter_list: Vec<K>,
}

impl<I, K> FilterCommon<I, K>
    where
        I: Iterator,
        K: PartialEq<I::Item>,
{
    fn remove_item(&mut self, item: &I::Item) -> bool {
        if let Some(index) = self.filter_list.iter().position(|x| x == item) {
            self.filter_list.remove(index);
            return true;
        }
        false
    }
}

/// Create a new **FilterCommon** iterator.
pub fn filter_common<I, K>(iterator: I, filter_list: Vec<K>) -> FilterCommon<I, K>
    where
        I: Iterator,
        K: PartialEq<I::Item>,
{
    FilterCommon {
        iterator,
        filter_list,
    }
}

impl<I, K> Iterator for FilterCommon<I, K>
    where
        I: Iterator,
        K: PartialEq<I::Item>,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let item = self.iterator.next()?;
            if !self.remove_item(&item) {
                return Some(item);
            }
        }
    }
}
