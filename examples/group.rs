
extern crate itertools;

use itertools::Itertools;

fn main() {
    let data = "abc12345abc".chars();

    // This example does something like .group_by().
    //
    // Instead of allocating, it just walks the iterator twice, and this
    // requires that the iterator is small and easy to clone, like for example
    // the slice or chars iterators.

    // using Itertools::batching
    //
    // Yield (key, iterator) for each run of identical keys.
    let key_func = |c: char| c.is_alphabetic();
    for (key, iter) in data.batching(|mut it| {
            let start = it.clone();
            match it.next() {
                None => return None,
                Some(elt) => {
                    let key = key_func(elt);
                    // using Itertools::take_while_ref
                    let n = 1 + it.take_while_ref(|elt| key_func(*elt) == key).count();
                    return Some((key, start.take(n)))
                }
            }
        })
    {
        for elt in iter {
            println!("Key={:?}, elt={:?}", key, elt);
        }
    }
}
