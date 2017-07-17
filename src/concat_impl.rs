use Itertools;

/// Concatenate all items of the iterator into a single extendable destination.
///
/// This combinator will extend the first item with the contents of the rest
/// of the items of the iterator. If the iterator is empty, the default value
/// will be returned.
pub fn concat<I: Iterator>(iter: I) -> I::Item
    where I: Iterator,
          I::Item: Extend<<<I as Iterator>::Item as IntoIterator>::Item> + IntoIterator + Default
{
    iter.fold1(|mut a, b| { a.extend(b); a }).unwrap_or_else(|| <_>::default())
}
