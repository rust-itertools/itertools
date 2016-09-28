extern crate itertools;

use itertools::Itertools;
use std::collections::VecDeque;

#[test]
fn tuples() {
    let v = [1, 2, 3, 4, 5];
    let mut iter = v.iter().cloned().tuples();
    assert_eq!(Some((1,)), iter.next());
    assert_eq!(Some((2,)), iter.next());
    assert_eq!(Some((3,)), iter.next());
    assert_eq!(Some((4,)), iter.next());
    assert_eq!(Some((5,)), iter.next());
    assert_eq!(None, iter.next());
    assert_eq!(Vec::<usize>::new(), iter.into_buffer());

    let mut iter = v.iter().cloned().tuples();
    assert_eq!(Some((1, 2)), iter.next());
    assert_eq!(Some((3, 4)), iter.next());
    assert_eq!(None, iter.next());
    assert_eq!(vec![5], iter.into_buffer());

    let mut iter = v.iter().cloned().tuples();
    assert_eq!(Some((1, 2, 3)), iter.next());
    assert_eq!(None, iter.next());
    assert_eq!(vec![4, 5], iter.into_buffer());

    let mut iter = v.iter().cloned().tuples();
    assert_eq!(Some((1, 2, 3, 4)), iter.next());
    assert_eq!(None, iter.next());
    assert_eq!(vec![5], iter.into_buffer());
}

#[test]
fn tuple_windows() {
    let empty = VecDeque::new();
    let v = [1, 2, 3, 4, 5];

    let mut iter = v.iter().cloned().tuple_windows();
    assert_eq!(Some((1,)), iter.next());
    assert_eq!(Some((2,)), iter.next());
    assert_eq!(Some((3,)), iter.next());
    assert_eq!(Some((4,)), iter.next());
    assert_eq!(Some((5,)), iter.next());
    assert_eq!(None, iter.next());
    assert_eq!(empty, iter.into_parts().0);

    let mut iter = v.iter().cloned().tuple_windows();
    assert_eq!(Some((1, 2)), iter.next());
    assert_eq!(Some((2, 3)), iter.next());
    assert_eq!(Some((3, 4)), iter.next());
    assert_eq!(Some((4, 5)), iter.next());
    assert_eq!(None, iter.next());
    assert_eq!(VecDeque::from(vec![5]), iter.into_parts().0);

    let mut iter = v.iter().cloned().tuple_windows();
    assert_eq!(Some((1, 2, 3)), iter.next());
    assert_eq!(Some((2, 3, 4)), iter.next());
    assert_eq!(Some((3, 4, 5)), iter.next());
    assert_eq!(None, iter.next());
    assert_eq!(VecDeque::from(vec![4, 5]), iter.into_parts().0);

    let mut iter = v.iter().cloned().tuple_windows();
    assert_eq!(Some((1, 2, 3, 4)), iter.next());
    assert_eq!(Some((2, 3, 4, 5)), iter.next());
    assert_eq!(None, iter.next());
    assert_eq!(VecDeque::from(vec![3, 4, 5]), iter.into_parts().0);

    let v = [1, 2, 3];
    let mut iter = v.iter().cloned().tuple_windows::<(_, _, _, _)>();
    assert_eq!(None, iter.next());
    assert_eq!(VecDeque::from(vec![1, 2, 3]), iter.into_parts().0);
}
