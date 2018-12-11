/// Implementation guts for `min_set`, `min_set_by`, and `min_set_by_key`.
pub fn min_set_impl<I, K, F, L>(mut it: I,
                                mut key_for: F,
                                mut lt: L) -> Option<Vec<I::Item>>
    where I: Iterator,
          F: FnMut(&I::Item) -> K,
          L: FnMut(&I::Item, &I::Item, &K, &K) -> bool,
{
    let (mut result, mut current_key) = match it.next() {
        None => return None,
        Some(element) => {
            let key = key_for(&element);
            (vec![element], key)
        }
    };

    for element in it {
        let key = key_for(&element);
        if lt(&element, &result[0], &key, &current_key) {
            result.clear();
            result.push(element);
            current_key = key;
        } else if !lt(&result[0], &element, &current_key, &key) {
            result.push(element);
        }
    }

    Some(result)
}

/// Implementation guts for `ax_set`, `max_set_by`, and `max_set_by_key`.
pub fn max_set_impl<I, K, F, L>(it: I,
                                key_for: F,
                                mut lt: L) -> Option<Vec<I::Item>>
    where I: Iterator,
          F: FnMut(&I::Item) -> K,
          L: FnMut(&I::Item, &I::Item, &K, &K) -> bool,
{
    min_set_impl(it, key_for, |it1, it2, key1, key2| lt(it2, it1, key2, key1))
}


