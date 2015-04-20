//! Licensed under the Apache License, Version 2.0
//! http://www.apache.org/licenses/LICENSE-2.0 or the MIT license
//! http://opensource.org/licenses/MIT, at your
//! option. This file may not be copied, modified, or distributed
//! except according to those terms.

#[macro_use]
extern crate itertools as it;

use std::fmt::Debug;
use it::Itertools;
use it::Interleave;
use it::Zip;

fn assert_iters_equal<A, I, J>(mut it: I, mut jt: J) where
    A: PartialEq + Debug,
    I: Iterator<Item=A>,
    J: Iterator<Item=A>,
{
    loop {
        let elti = it.next();
        let eltj = jt.next();
        assert_eq!(elti, eltj);
        if elti.is_none() { break; }
    }
}

#[test]
fn product2() {
    let s = "αβ";

    let mut prod = iproduct!(s.chars(), 0..2);
    assert!(prod.next() == Some(('α', 0)));
    assert!(prod.next() == Some(('α', 1)));
    assert!(prod.next() == Some(('β', 0)));
    assert!(prod.next() == Some(('β', 1)));
    assert!(prod.next() == None);
}

#[test]
fn product3() {
    let prod = iproduct!(0..3, 0..2, 0..2);
    assert_eq!(prod.size_hint(), (12, Some(12)));
    let v = prod.collect_vec();
    for i in 0..3 {
        for j in 0..2 {
            for k in 0..2 {
                assert!((i, j, k) == v[(i * 2 * 2 + j * 2 + k) as usize]);
            }
        }
    }
    for (_, _, _, _) in iproduct!(0..3, 0..2, 0..2, 0..3) {
        /* test compiles */
    }
}

#[test]
fn izip3() {
    let mut zip = Zip::new((0..3, 0..2, 0..2i8));
    for i in 0..2 {
        assert!((i as usize, i, i as i8) == zip.next().unwrap());
    }
    assert!(zip.next().is_none());

    
    let xs: [isize; 0] = [];
    let mut zip = Zip::new((0..3, 0..2, 0..2i8, xs.iter()));
    assert!(zip.next().is_none());

    for (_, _, _, _) in Zip::new((0..3, 0..2, 0..2, 0..3)) {
        /* test compiles */
    }
}

#[test]
fn write_to() {
    let xs = [7, 9, 8];
    let mut ys = [0; 5];
    let cnt = it::write(ys.iter_mut(), xs.iter().map(|x| *x));
    assert!(cnt == xs.len());
    assert!(ys == [7, 9, 8, 0, 0]);

    let cnt = it::write(ys.iter_mut(), 0..10);
    assert!(cnt == ys.len());
    assert!(ys == [0, 1, 2, 3, 4]);
}

#[test]
fn interleave() {
    let xs: [u8; 0]  = [];
    let ys = [7u8, 9, 8, 10];
    let zs = [2u8, 77];
    let it = Interleave::new(xs.iter(), ys.iter());
    assert_iters_equal(it, ys.iter());

    let rs = [7u8, 2, 9, 77, 8, 10];
    let it = Interleave::new(ys.iter(), zs.iter());
    assert_iters_equal(it, rs.iter());
}

#[test]
fn times() {
    assert!(it::times(0).count() == 0);
    assert!(it::times(5).count() == 5);
}

#[test]
fn drain() {
    let xs = [1i32,2,3];
    let mut sum = 0;
    xs.iter().map(|elt| sum += *elt).drain();
    assert!(sum == 6);
}

#[test]
fn foreach() {
    let xs = [1i32, 2, 3];
    let mut sum = 0;
    xs.iter().foreach(|elt| sum += *elt);
    assert!(sum == 6);
}

#[test]
fn dropn() {
    let xs = [1, 2, 3];
    let mut it = xs.iter();
    assert!(it.dropn(2) == 2);
    assert!(it.next().is_some());
    assert!(it.next().is_none());
    let mut it = xs.iter();
    assert!(it.dropn(5) == 3);
    assert!(it.next().is_none());
}

#[test]
fn dropping() {
    let xs = [1, 2, 3];
    let mut it = xs.iter().dropping(2);
    assert!(it.next().is_some());
    assert!(it.next().is_none());
    let mut it = xs.iter().dropping(5);
    assert!(it.next().is_none());
}

#[test]
fn intersperse() {
    let xs = ["a", "", "b", "c"];
    let v: Vec<&str> = xs.iter().map(|x| x.clone()).intersperse(", ").collect();
    let text: String = v.concat();
    assert_eq!(text, "a, , b, c".to_string());

    let ys = [0, 1, 2, 3];
    let mut it = ys[..0].iter().map(|x| *x).intersperse(1);
    assert!(it.next() == None);
}

#[test]
fn linspace() {
    let mut iter = it::linspace::<f32>(0., 2., 3);
    assert_eq!(iter.next(), Some(0.0));
    assert_eq!(iter.next(), Some(1.0));
    assert_eq!(iter.next(), Some(2.0));
    assert_eq!(iter.next(), None);

    let mut iter = it::linspace::<f32>(0., -2., 4);
    assert_eq!(iter.next(), Some(0.));
    assert_eq!(iter.next(), Some(-0.666666666667));
    assert_eq!(iter.next(), Some(-1.333333333333));
    assert_eq!(iter.next(), Some(-2.));
    assert_eq!(iter.next(), None);

    let mut iter = it::linspace::<f32>(0., 1., 1);
    assert_eq!(iter.next(), Some(0.));
    assert_eq!(iter.next(), None);

    let mut iter = it::linspace::<f32>(0., 1., 0);
    assert_eq!(iter.next(), None);
}

#[test]
fn dedup() {
    let xs = [0, 1, 1, 1, 2, 1, 3, 3];
    let ys = [0, 1, 2, 1, 3];
    assert_iters_equal(ys.iter(), xs.iter().dedup());
    let xs = [0, 0, 0, 0, 0];
    let ys = [0];
    assert_iters_equal(ys.iter(), xs.iter().dedup());
}

#[test]
fn batching() {
    let xs = [0, 1, 2, 1, 3];
    let ys = [(0, 1), (2, 1)];

    // An iterator that gathers elements up in pairs
    let pit = xs.iter().cloned().batching(|mut it| {
               match it.next() {
                   None => None,
                   Some(x) => match it.next() {
                       None => None,
                       Some(y) => Some((x, y)),
                   }
               }
           });
    assert_iters_equal(pit, ys.iter().cloned());
}

#[test]
fn group_by() {
    let xs = [0, 1, 1, 1, 2, 1, 3, 3];
    let ans = vec![(0, vec![0]), (1, vec![1, 1, 1]),
                   (2, vec![2]), (1, vec![1]), (3, vec![3, 3])];
    
    let gb = xs.iter().cloned().group_by(|elt| *elt);
    assert_iters_equal(gb, ans.into_iter());
}

#[test]
fn put_back() {
    let xs = [0, 1, 1, 1, 2, 1, 3, 3];
    let mut pb = it::PutBack::new(xs.iter().cloned());
    pb.next();
    pb.put_back(1);
    pb.put_back(0);
    assert_iters_equal(pb, xs.iter().cloned());
}

#[test]
fn tee() {
    let xs  = [0, 1, 2, 3];
    let (mut t1, mut t2) = xs.iter().cloned().tee();
    assert_eq!(t1.next(), Some(0));
    assert_eq!(t2.next(), Some(0));
    assert_eq!(t1.next(), Some(1));
    assert_eq!(t1.next(), Some(2));
    assert_eq!(t1.next(), Some(3));
    assert_eq!(t1.next(), None);
    assert_eq!(t2.next(), Some(1));
    assert_eq!(t2.next(), Some(2));
    assert_eq!(t1.next(), None);
    assert_eq!(t2.next(), Some(3));
    assert_eq!(t2.next(), None);
    assert_eq!(t1.next(), None);
    assert_eq!(t2.next(), None);

    let (t1, t2) = xs.iter().cloned().tee();
    assert_iters_equal(t1, xs.iter().cloned());
    assert_iters_equal(t2, xs.iter().cloned());

    let (t1, t2) = xs.iter().cloned().tee();
    assert_iters_equal(t1.zip(t2), xs.iter().cloned().zip(xs.iter().cloned()));
}


#[test]
fn rciter() {
    let xs = [0, 1, 1, 1, 2, 1, 3, 5, 6];

    let mut r1 = xs.iter().cloned().into_rc();
    let mut r2 = r1.clone();
    assert_eq!(r1.next(), Some(0));
    assert_eq!(r2.next(), Some(1));
    let mut z = r1.zip(r2);
    assert_eq!(z.next(), Some((1, 1)));
    assert_eq!(z.next(), Some((2, 1)));
    assert_eq!(z.next(), Some((3, 5)));
    assert_eq!(z.next(), None);
}

#[test]
fn slice() {
    assert_iters_equal((0..10).slice(..3), 0..3);
    assert_iters_equal((0..10).slice(3..7), 3..7);
    assert_iters_equal((0..10).slice(3..27), 3..10);
    assert_iters_equal((0..10).slice(44..), 0..0);
}

#[test]
fn step() {
    assert_iters_equal((0..10).step(1), (0..10));
    assert_iters_equal((0..10).step(2), (0..10).filter(|x: &i32| *x % 2 == 0));
    assert_iters_equal((0..10).step(10), 0..1);
}

#[test]
fn trait_pointers() {
    struct ByRef<'r, I: ?Sized>(&'r mut I) where I: 'r;

    impl<'r, X, I: ?Sized> Iterator for ByRef<'r, I> where
        I: 'r + Iterator<Item=X>
    {
        type Item = X;
        fn next(&mut self) -> Option<X>
        {
            self.0.next()
        }
    }

    let mut it = Box::new(0..10) as Box<Iterator<Item=i32>>;
    assert_eq!(it.next(), Some(0));

    {
        /* make sure foreach works on non-Sized */
        let mut jt: &mut Iterator<Item=i32> = &mut *it;
        assert_eq!(jt.next(), Some(1));

        {
            let mut r = ByRef(jt);
            assert_eq!(r.next(), Some(2));
        }

        assert_eq!(jt.find_position(|x| *x == 4), Some((1, 4)));
        jt.foreach(|_| ());
    }
}

#[test]
fn merge() {
    assert_iters_equal((0..10).step(2).merge((1..10).step(2)), (0..10));
}

#[test]
fn merge_by() {
    let odd : Vec<(u32, &str)> = vec![(1, "hello"), (3, "world"), (5, "!")];
    let even = vec![(2, "foo"), (4, "bar"), (6, "baz")];
    let expected = vec![(1, "hello"), (2, "foo"), (3, "world"), (4, "bar"), (5, "!"), (6, "baz")];
    let results = odd.iter().merge_by(even.iter(), |a, b|{ a.0.cmp(&b.0)});
    assert_iters_equal(results, expected.iter());
}

#[test]
fn merge_by_btree() {
    use std::collections::BTreeMap;
    let mut bt1 = BTreeMap::new();
    bt1.insert("hello", 1);
    bt1.insert("world", 3);
    let mut bt2 = BTreeMap::new();
    bt2.insert("foo", 2);
    bt2.insert("bar", 4);
    let results = bt1.into_iter().merge_by(bt2.into_iter(), |a, b|{a.0.cmp(&b.0)});
    let expected = vec![("bar", 4), ("foo", 2), ("hello", 1), ("world", 3)];
    assert_iters_equal(results, expected.into_iter());
}

#[test]
fn to_string_join() {
    let many = [1, 2, 3];
    let one  = [1];
    let none: Vec<i32> = vec![];

    assert_eq!(many.iter().to_string_join(", "), "1, 2, 3");
    assert_eq!( one.iter().to_string_join(", "), "1");
    assert_eq!(none.iter().to_string_join(", "), "");
}

#[test]
fn multipeek() {
    let nums = vec![1u8,2,3,4,5];

    let multipeek = nums.iter().map(|&x| x).multipeek();
    assert_eq!(nums, multipeek.collect::<Vec<_>>());

    let mut multipeek = nums.iter().map(|&x| x).multipeek();
    assert_eq!(multipeek.peek(), Some(&1));
    assert_eq!(multipeek.next(), Some(1));
    assert_eq!(multipeek.peek(), Some(&2));
    assert_eq!(multipeek.peek(), Some(&3));
    assert_eq!(multipeek.next(), Some(2));
    assert_eq!(multipeek.peek(), Some(&3));
    assert_eq!(multipeek.peek(), Some(&4));
    assert_eq!(multipeek.peek(), Some(&5));
    assert_eq!(multipeek.peek(), None);
    assert_eq!(multipeek.next(), Some(3));
    assert_eq!(multipeek.next(), Some(4));
    assert_eq!(multipeek.next(), Some(5));
    assert_eq!(multipeek.next(), None);
    assert_eq!(multipeek.peek(), None);

}

#[test]
fn repeatn() {
    let s = "α";
    let mut it = it::RepeatN::new(s, 3);
    assert_eq!(it.len(), 3);
    assert_eq!(it.next(), Some(s));
    assert_eq!(it.next(), Some(s));
    assert_eq!(it.next(), Some(s));
    assert_eq!(it.next(), None);
    assert_eq!(it.next(), None);
}

#[test]
fn count_clones() {
    // Check that RepeatN only clones N - 1 times.

    use std::cell::Cell;
    #[derive(PartialEq, Debug)]
    struct Foo {
        n: Cell<usize>
    }

    impl Clone for Foo
    {
        fn clone(&self) -> Self
        {
            let n = self.n.get();
            self.n.set(n + 1);
            Foo { n: Cell::new(n + 1) }
        }
    }


    for n in 0..10 {
        let f = Foo{n: Cell::new(0)};
        let mut it = it::RepeatN::new(f, n);
        // drain it
        let last = it.last();
        if n == 0 {
            assert_eq!(last, None);
        } else {
            assert_eq!(last, Some(Foo{n: Cell::new(n - 1)}));
        }
    }
}
