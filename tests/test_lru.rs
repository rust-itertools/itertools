use itertools as it;
use crate::it::Itertools;

#[cfg(feature = "use_lru")]
#[test]
fn unique_by_lru() {
    let input = ["1", "2", "3", "11", "22", "33", "44", "111", "222", "333", "444"].iter();
    let actual = input.unique_by_lru(3, |x| x.chars().nth(0));
    let expected = ["1", "2", "3", "44", "111", "222", "333", "444"].iter();
    it::assert_equal(actual, expected);
}

#[cfg(feature = "use_lru")]
#[test]
fn unique_lru() {
    let input = [1, 2, 3, 1, 2, 3, 4, 1, 2, 3, 4].iter();
    let actual = input.unique_lru(3);
    let expected = [1, 2, 3, 4, 1, 2, 3, 4].iter();
    it::assert_equal(actual, expected);
}

