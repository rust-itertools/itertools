use itertools::Itertools;

#[test]
fn arrays() {
    let v = [1, 2, 3, 4, 5];
    let mut iter = v.iter().cloned().arrays();
    assert_eq!(Some([1]), iter.next());
    assert_eq!(Some([2]), iter.next());
    assert_eq!(Some([3]), iter.next());
    assert_eq!(Some([4]), iter.next());
    assert_eq!(Some([5]), iter.next());
    assert_eq!(None, iter.next());
    assert_eq!(Vec::<i32>::new(), iter.remaining());

    let mut iter = v.iter().cloned().arrays();
    assert_eq!(Some([1, 2]), iter.next());
    assert_eq!(Some([3, 4]), iter.next());
    assert_eq!(None, iter.next());
    assert_eq!(vec![5], iter.remaining());

    let mut iter = v.iter().cloned().arrays();
    assert_eq!(Some([1, 2, 3]), iter.next());
    assert_eq!(None, iter.next());
    assert_eq!(vec![4, 5], iter.remaining());

    let mut iter = v.iter().cloned().arrays();
    assert_eq!(Some([1, 2, 3, 4]), iter.next());
    assert_eq!(None, iter.next());
    assert_eq!(vec![5], iter.remaining());
}

#[test]
fn next_array() {
    let v = [1, 2, 3, 4, 5];
    let mut iter = v.iter();
    assert_eq!(iter.next_array().map(|[&x, &y]| (x, y)), Some((1, 2)));
    assert_eq!(iter.next_array().map(|[&x, &y]| (x, y)), Some((3, 4)));
    assert_eq!(iter.next_array::<2>(), None);
}
