use itertools::Itertools;
use itertools::{put_back, put_back_n};

#[test]
fn peeking_map_while_peekable() {
    let vec = vec!["0", "1", "2", "three", "four"];
    let mut xs = vec.iter().peekable();
    let ys: Vec<u8> = xs.peeking_map_while(|x| x.parse().ok()).collect();

    assert_eq!(ys, vec![0, 1, 2]);
    assert_eq!(xs.next(), Some(&"three"));
}

#[test]
fn peeking_map_while_put_back() {
    let mut r = put_back(vec!["0", "1", "2", "three", "four"]);
    r.peeking_map_while(|x| x.parse::<usize>().ok()).count();
    assert_eq!(r.next(), Some("three"));
    r.peeking_map_while(|_| Some(())).count();
    assert_eq!(r.next(), None);
}

#[test]
fn peeking_map_while_put_back_n() {
    let mut r = put_back_n(vec!["1", "2", "three", "four"]);
    for elt in vec!["zero"].iter().rev() {
        r.put_back(elt);
    }
    r.peeking_map_while(|x| x.parse::<usize>().ok()).count();
    assert_eq!(r.next(), Some("zero"));
    r.peeking_map_while(|_| Some(())).count();
    assert_eq!(r.next(), None);
}

#[test]
fn peeking_map_while_slice_iter() {
    let v = [1, 2, 3, 4, 5, 6];
    let mut r = v.iter();
    r.peeking_map_while(|x| if **x <= 3 { Some(**x) } else { None })
        .count();
    assert_eq!(r.next(), Some(&4));
    r.peeking_map_while(|_| Some(())).count();
    assert_eq!(r.next(), None);
}

#[test]
fn peeking_map_while_slice_iter_rev() {
    let v = [1, 2, 3, 4, 5, 6];
    let mut r = v.iter().rev();
    r.peeking_map_while(|x| if **x >= 3 { Some(*x) } else { None })
        .count();
    assert_eq!(r.next(), Some(&2));
    r.peeking_map_while(|_| Some(())).count();
    assert_eq!(r.next(), None);
}
