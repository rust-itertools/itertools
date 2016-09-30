extern crate itertools;

use itertools::Itertools;

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
    assert_eq!(None, iter.into_buffer().next());

    let mut iter = v.iter().cloned().tuples();
    assert_eq!(Some((1, 2)), iter.next());
    assert_eq!(Some((3, 4)), iter.next());
    assert_eq!(None, iter.next());
    itertools::assert_equal(vec![5], iter.into_buffer());

    let mut iter = v.iter().cloned().tuples();
    assert_eq!(Some((1, 2, 3)), iter.next());
    assert_eq!(None, iter.next());
    itertools::assert_equal(vec![4, 5], iter.into_buffer());

    let mut iter = v.iter().cloned().tuples();
    assert_eq!(Some((1, 2, 3, 4)), iter.next());
    assert_eq!(None, iter.next());
    itertools::assert_equal(vec![5], iter.into_buffer());
}

#[test]
fn tuple_windows() {
    let v = [1, 2, 3, 4, 5];

    let mut iter = v.iter().cloned().tuple_windows();
    assert_eq!(Some((1,)), iter.next());
    assert_eq!(Some((2,)), iter.next());
    assert_eq!(Some((3,)), iter.next());
    let (mut buffer, mut it) = iter.into_parts();
    assert_eq!(None, buffer.next());
    assert_eq!(Some(4), it.next());

    let mut iter = v.iter().cloned().tuple_windows();
    assert_eq!(Some((1, 2)), iter.next());
    assert_eq!(Some((2, 3)), iter.next());
    assert_eq!(Some((3, 4)), iter.next());
    assert_eq!(Some((4, 5)), iter.next());
    assert_eq!(None, iter.next());
    let (mut buffer, mut it) = iter.into_parts();
    assert_eq!(None, buffer.next());
    assert_eq!(None, it.next());

    let mut iter = v.iter().cloned().tuple_windows();
    assert_eq!(Some((1, 2, 3)), iter.next());
    assert_eq!(Some((2, 3, 4)), iter.next());
    assert_eq!(Some((3, 4, 5)), iter.next());
    assert_eq!(None, iter.next());
    let (mut buffer, mut it) = iter.into_parts();
    assert_eq!(None, buffer.next());
    assert_eq!(None, it.next());

    let mut iter = v.iter().cloned().tuple_windows();
    assert_eq!(Some((1, 2, 3, 4)), iter.next());
    assert_eq!(Some((2, 3, 4, 5)), iter.next());
    assert_eq!(None, iter.next());
    let (mut buffer, mut it) = iter.into_parts();
    assert_eq!(None, buffer.next());
    assert_eq!(None, it.next());

    let v = [1, 2, 3];
    let mut iter = v.iter().cloned().tuple_windows::<(_, _, _, _)>();
    assert_eq!(None, iter.next());
    let (buffer, mut it) = iter.into_parts();
    itertools::assert_equal(vec![1, 2, 3], buffer);
    assert_eq!(None, it.next());
}
