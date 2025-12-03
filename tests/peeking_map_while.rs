use either::Either;
use itertools::{put_back, Itertools};

#[test]
fn peeking_map_while_peekable() {
    let mut r = (0..10).peekable();
    let last = r
        .peeking_map_while(|x| match x {
            0..3 => Either::Left(x * 2),
            _ => Either::Right(x),
        })
        .last();
    assert_eq!(last, Some(4));
    assert_eq!(r.next(), Some(3));
}

#[test]
fn peeking_map_while_put_back() {
    let mut r = put_back(0..10);
    let last = r
        .peeking_map_while(|x| match x {
            0..3 => Either::Left(x * 2),
            _ => Either::Right(x),
        })
        .last();
    assert_eq!(last, Some(4));
    assert_eq!(r.next(), Some(3));
}
