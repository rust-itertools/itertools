#![cfg(feature = "quickcheck")]

//! The purpose of these tests is to cover corner cases of iterators
//! and adaptors.
//!
//! In particular we test the tedious size_hint and exact size correctness.

#[macro_use] extern crate itertools;

extern crate quickcheck;

use std::default::Default;

use quickcheck as qc;
use std::ops::Range;
use itertools::Itertools;
use itertools::{
    Zip,
    Stride,
    EitherOrBoth,
};
use itertools::free::{
    zip,
    zip_eq,
};

/// Our base iterator that we can impl Arbitrary for
///
/// NOTE: Iter is tricky and is not fused, to help catch bugs.
/// At the end it will return None once, then return Some(0),
/// then return None again.
#[derive(Clone, Debug)]
struct Iter<T>(Range<T>, i32); // with fuse/done flag

impl<T> Iter<T>
{
    fn new(it: Range<T>) -> Self
    {
        Iter(it, 0)
    }
}

impl<T> Iterator for Iter<T> where Range<T>: Iterator,
    <Range<T> as Iterator>::Item: Default,
{
    type Item = <Range<T> as Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item>
    {
        let elt = self.0.next();
        if elt.is_none() {
            self.1 += 1;
            // check fuse flag
            if self.1 == 2 {
                return Some(Default::default())
            }
        }
        elt
    }

    fn size_hint(&self) -> (usize, Option<usize>)
    {
        self.0.size_hint()
    }
}

impl<T> DoubleEndedIterator for Iter<T> where Range<T>: DoubleEndedIterator,
    <Range<T> as Iterator>::Item: Default,
{
    fn next_back(&mut self) -> Option<Self::Item> { self.0.next_back() }
}

impl<T> ExactSizeIterator for Iter<T> where Range<T>: ExactSizeIterator,
    <Range<T> as Iterator>::Item: Default,
{ }

impl<T> qc::Arbitrary for Iter<T> where T: qc::Arbitrary
{
    fn arbitrary<G: qc::Gen>(g: &mut G) -> Self
    {
        Iter::new(T::arbitrary(g)..T::arbitrary(g))
    }

    fn shrink(&self) -> Box<Iterator<Item=Iter<T>>>
    {
        let r = self.0.clone();
        Box::new(
            r.start.shrink().flat_map(move |x| {
                r.end.shrink().map(move |y| (x.clone(), y))
            })
            .map(|(a, b)| Iter::new(a..b))
        )
    }
}

fn correct_size_hint<I: Iterator>(mut it: I) -> bool {
    // record size hint at each iteration
    let initial_hint = it.size_hint();
    let mut hints = Vec::with_capacity(initial_hint.0 + 1);
    hints.push(initial_hint);
    while let Some(_) = it.next() {
        hints.push(it.size_hint())
    }

    let mut true_count = hints.len(); // start off +1 too much

    // check all the size hints
    for &(low, hi) in &hints {
        true_count -= 1;
        if low > true_count ||
            (hi.is_some() && hi.unwrap() < true_count)
        {
            println!("True size: {:?}, size hint: {:?}", true_count, (low, hi));
            //println!("All hints: {:?}", hints);
            return false
        }
    }
    true
}

fn exact_size<I: ExactSizeIterator>(mut it: I) -> bool {
    // check every iteration
    let (mut low, mut hi) = it.size_hint();
    if Some(low) != hi { return false; }
    while let Some(_) = it.next() {
        let (xlow, xhi) = it.size_hint();
        if low != xlow + 1 { return false; }
        low = xlow;
        hi = xhi;
        if Some(low) != hi { return false; }
    }
    let (low, hi) = it.size_hint();
    low == 0 && hi == Some(0)
}

/*
 * NOTE: Range<i8> is broken!
 * (all signed ranges are)
#[quickcheck]
fn size_range_i8(a: Iter<i8>) -> bool {
    exact_size(a)
}

#[quickcheck]
fn size_range_i16(a: Iter<i16>) -> bool {
    exact_size(a)
}

#[quickcheck]
fn size_range_u8(a: Iter<u8>) -> bool {
    exact_size(a)
}
 */

macro_rules! quickcheck {
    ($name:ident(1), $func:item) => {
        #[test]
        fn $name() {
            $func
            quickcheck::quickcheck(prop as fn(_) -> _);
        }
    };
    ($name:ident(2), $func:item) => {
        #[test]
        fn $name() {
            $func
            quickcheck::quickcheck(prop as fn(_, _) -> _);
        }
    };
    ($name:ident(3), $func:item) => {
        #[test]
        fn $name() {
            $func
            quickcheck::quickcheck(prop as fn(_, _, _) -> _);
        }
    };
    ($name:ident(4), $func:item) => {
        #[test]
        fn $name() {
            $func
            quickcheck::quickcheck(prop as fn(_, _, _, _) -> _);
        }
    };
}

quickcheck! {
    size_stride(2),
    fn prop(data: Vec<u8>, mut stride: isize) -> bool {
        if stride == 0 {
            stride += 1; // never zero
        }
        exact_size(Stride::from_slice(&data, stride))
    }
}

quickcheck! {
    equal_stride(2),
    fn prop(data: Vec<u8>, mut stride: i8) -> bool {
        if stride == 0 {
            // never zero
            stride += 1;
        }
        if stride > 0 {
            itertools::equal(Stride::from_slice(&data, stride as isize),
                             data.iter().step(stride as usize))
        } else {
            itertools::equal(Stride::from_slice(&data, stride as isize),
                             data.iter().rev().step(-stride as usize))
        }
    }
}

quickcheck! {
    size_product(2),
    fn prop(a: Iter<u16>, b: Iter<u16>) -> bool {
        correct_size_hint(a.cartesian_product(b))
    }
}

quickcheck! {
    size_product3(3),
    fn prop(a: Iter<u16>, b: Iter<u16>, c: Iter<u16>) -> bool {
        correct_size_hint(iproduct!(a, b, c))
    }
}

quickcheck! {
    size_step(2),
    fn prop(a: Iter<i16>, mut s: usize) -> bool {
        if s == 0 {
            s += 1; // never zero
        }
        let filt = a.clone().dedup();
        correct_size_hint(filt.step(s)) &&
            exact_size(a.step(s))
    }
}

quickcheck! {
    size_multipeek(2),
    fn prop(a: Iter<u16>, s: u8) -> bool {
        let mut it = a.multipeek();
        // peek a few times
        for _ in 0..s {
            it.peek();
        }
        exact_size(it)
    }
}

quickcheck! {
    equal_merge(2),
    fn prop(a: Vec<i16>, b: Vec<i16>) -> bool {
        let mut sa = a.clone();
        let mut sb = b.clone();
        sa.sort();
        sb.sort();
        let mut merged = sa.clone();
        merged.extend(sb.iter().cloned());
        merged.sort();
        itertools::equal(&merged, sa.iter().merge(&sb))
    }

}

quickcheck! {
    size_merge(2),
    fn prop(a: Iter<u16>, b: Iter<u16>) -> bool {
        correct_size_hint(a.merge(b))
    }
}

quickcheck! {
    size_zip(3),
    fn prop(a: Iter<i16>, b: Iter<i16>, c: Iter<i16>) -> bool {
        let filt = a.clone().dedup();
        correct_size_hint(Zip::new((filt, b.clone(), c.clone()))) &&
            exact_size(Zip::new((a, b, c)))
    }
}

quickcheck! {
    size_zip_rc(2),
    fn prop(a: Iter<i16>, b: Iter<i16>) -> bool {
        let rc = a.clone().into_rc();
        correct_size_hint(Zip::new((&rc, &rc, b)))
    }
}

quickcheck! {
    equal_kmerge(3),
    fn prop(a: Vec<i16>, b: Vec<i16>, c: Vec<i16>) -> bool {
        use itertools::free::kmerge;
        let mut sa = a.clone();
        let mut sb = b.clone();
        let mut sc = c.clone();
        sa.sort();
        sb.sort();
        sc.sort();
        let mut merged = sa.clone();
        merged.extend(sb.iter().cloned());
        merged.extend(sc.iter().cloned());
        merged.sort();
        itertools::equal(merged.into_iter(), kmerge(vec![sa, sb, sc]))
    }
}

quickcheck! {
    size_kmerge(3),
    fn prop(a: Iter<i16>, b: Iter<i16>, c: Iter<i16>) -> bool {
        use itertools::free::kmerge;
        correct_size_hint(kmerge(vec![a, b, c]))
    }
}

quickcheck! {
    equal_zip_eq(2),
    fn prop(a: Vec<i32>, b: Vec<i32>) -> bool {
        let len = std::cmp::min(a.len(), b.len());
        let a = &a[..len];
        let b = &b[..len];
        itertools::equal(zip_eq(a, b), zip(a, b))
    }
}

quickcheck! {
    size_zip_longest(2),
    fn prop(a: Iter<i16>, b: Iter<i16>) -> bool {
        let filt = a.clone().dedup();
        let filt2 = b.clone().dedup();
        correct_size_hint(filt.zip_longest(b.clone())) &&
        correct_size_hint(a.clone().zip_longest(filt2)) &&
            exact_size(a.zip_longest(b))
    }
}

quickcheck! {
    size_2_zip_longest(2),
    fn prop(a: Iter<i16>, b: Iter<i16>) -> bool {
        let it = a.clone().zip_longest(b.clone());
        let jt = a.clone().zip_longest(b.clone());
        itertools::equal(a.clone(),
                         it.filter_map(|elt| match elt {
                             EitherOrBoth::Both(x, _) => Some(x),
                             EitherOrBoth::Left(x) => Some(x),
                             _ => None,
                         }
                         ))
            &&
        itertools::equal(b.clone(),
                         jt.filter_map(|elt| match elt {
                             EitherOrBoth::Both(_, y) => Some(y),
                             EitherOrBoth::Right(y) => Some(y),
                             _ => None,
                         }
                         ))
    }
}

quickcheck! {
    equal_islice(3),
    fn prop(a: Vec<i16>, x: usize, y: usize) -> bool {
        if x > y || y > a.len() { return true; }
        let slc = &a[x..y];
        itertools::equal(a.iter().slice(x..y), slc)
    }
}

quickcheck! {
    size_islice(3),
    fn prop(a: Iter<i16>, x: usize, y: usize) -> bool {
        correct_size_hint(a.clone().dedup().slice(x..y)) &&
            exact_size(a.clone().slice(x..y))
    }
}

quickcheck! {
    size_interleave(2),
    fn prop(a: Iter<i16>, b: Iter<i16>) -> bool {
        correct_size_hint(a.interleave(b))
    }
}

quickcheck! {
    size_interleave_shortest(2),
    fn prop(a: Iter<i16>, b: Iter<i16>) -> bool {
        correct_size_hint(a.interleave_shortest(b))
    }
}

quickcheck! {
    size_intersperse(2),
    fn prop(a: Iter<i16>, x: i16) -> bool {
        correct_size_hint(a.intersperse(x))
    }
}

quickcheck! {
    equal_intersperse(2),
    fn prop(a: Vec<i32>, x: i32) -> bool {
        let mut inter = false;
        let mut i = 0;
        for elt in a.iter().cloned().intersperse(x) {
            if inter {
                if elt != x { return false }
            } else {
                if elt != a[i] { return false }
                i += 1;
            }
            inter = !inter;
        }
        true
    }
}

quickcheck! {
    equal_dedup(1),
    fn prop(a: Vec<i32>) -> bool {
        let mut b = a.clone();
        b.dedup();
        itertools::equal(&b, a.iter().dedup())
    }
}

quickcheck! {
    size_dedup(1),
    fn prop(a: Vec<i32>) -> bool {
        correct_size_hint(a.iter().dedup())
    }
}

quickcheck! {
    size_group_by(1),
    fn prop(a: Vec<i8>) -> bool {
        correct_size_hint(a.iter().group_by(|x| x.abs()))
    }
}

quickcheck! {
    size_linspace(3),
    fn prop(a: f32, b: f32, n: usize) -> bool {
        let it = itertools::linspace(a, b, n);
        it.len() == n &&
            exact_size(it)
    }
}

quickcheck! {
    exact_repeatn(2),
    fn prop(n: usize, x: i32) -> bool {
        let it = itertools::RepeatN::new(x, n);
        exact_size(it)
    }
}

#[cfg(feature = "unstable")]
quickcheck! {
    size_ziptrusted(2),
    fn prop(a: Vec<u8>, b: Vec<u8>) -> bool {
        exact_size(itertools::ZipTrusted::new((a.iter(), b.iter())))
    }
}

#[cfg(feature = "unstable")]
quickcheck! {
    size_ziptrusted3(3),
    fn prop(a: Vec<u8>, b: Vec<u8>, c: Vec<u8>) -> bool {
        exact_size(itertools::ZipTrusted::new((a.iter(), b.iter(), c.iter())))
    }
}

#[cfg(feature = "unstable")]
quickcheck! {
    equal_ziptrusted_mix(4),
    fn prop(a: Vec<u8>, b: Vec<()>, x: u8, y: u8) -> bool {
        let it = itertools::ZipTrusted::new((a.iter(), b.iter(), x..y));
        let jt = Zip::new((a.iter(), b.iter(), x..y));
        itertools::equal(it, jt)
    }
}

#[cfg(feature = "unstable")]
quickcheck! {
    size_ziptrusted_mix(4),
    fn prop(a: Vec<u8>, b: Vec<()>, x: u8, y: u8) -> bool {
        exact_size(itertools::ZipTrusted::new((a.iter(), b.iter(), x..y)))
    }
}

quickcheck! {
    size_put_back(2),
    fn prop(a: Vec<u8>, x: Option<u8>) -> bool {
        let mut it = itertools::PutBack::new(a.into_iter());
        match x {
            Some(t) => it.put_back(t),
            None => {}
        }
        correct_size_hint(it)
    }
}

quickcheck! {
    size_put_backn(2),
    fn prop(a: Vec<u8>, b: Vec<u8>) -> bool {
        let mut it = itertools::PutBackN::new(a.into_iter());
        for elt in b {
            it.put_back(elt)
        }
        correct_size_hint(it)
    }
}

quickcheck! {
    size_tee(1),
    fn prop(a: Vec<u8>) -> bool {
        let (mut t1, mut t2) = a.iter().tee();
        t1.next();
        t1.next();
        t2.next();
        exact_size(t1) && exact_size(t2)
    }
}

quickcheck! {
    size_tee_2(1),
    fn prop(a: Vec<u8>) -> bool {
        let (mut t1, mut t2) = a.iter().dedup().tee();
        t1.next();
        t1.next();
        t2.next();
        correct_size_hint(t1) && correct_size_hint(t2)
    }
}

quickcheck! {
    size_mend_slices(2),
    fn prop(a: Vec<u8>, splits: Vec<usize>) -> bool {
        let slice_iter = splits.into_iter().map(|ix|
            if ix < a.len() {
                &a[ix..(ix + 1)]
            } else {
                &a[0..0]
            }
        ).mend_slices();
        correct_size_hint(slice_iter)
    }
}

quickcheck! {
    size_take_while_ref(2),
    fn prop(a: Vec<u8>, stop: u8) -> bool {
        correct_size_hint(a.iter().take_while_ref(|x| **x != stop))
    }
}

quickcheck! {
    equal_partition(1),
    fn prop(mut a: Vec<i32>) -> bool {
        let mut ap = a.clone();
        let split_index = itertools::partition(&mut ap, |x| *x >= 0);
        let parted = (0..split_index).all(|i| ap[i] >= 0) &&
            (split_index..a.len()).all(|i| ap[i] < 0);

        a.sort();
        ap.sort();
        parted && (a == ap)
    }
}

quickcheck! {
    size_combinations(1),
    fn prop(it: Iter<i16>) -> bool {
        correct_size_hint(it.combinations())
    }
}

quickcheck! {
    equal_combinations(1),
    fn prop(it: Iter<i16>) -> bool {
        let values = it.clone().collect_vec();
        let mut cmb = it.combinations();
        for i in 0..values.len() {
            for j in i+1..values.len() {
                let pair = (values[i], values[j]);
                if pair != cmb.next().unwrap() {
                    return false;
                }
            }
        }
        cmb.next() == None
    }
}

quickcheck! {
    size_pad_tail(2),
    fn prop(it: Iter<i8>, pad: u8) -> bool {
        correct_size_hint(it.clone().pad_using(pad as usize, |_| 0)) &&
            correct_size_hint(it.dropping(1).rev().pad_using(pad as usize, |_| 0))
    }
}

quickcheck! {
    size_pad_tail2(2),
    fn prop(it: Iter<i8>, pad: u8) -> bool {
        exact_size(it.pad_using(pad as usize, |_| 0))
    }
}

quickcheck! {
    size_unique(1),
    fn prop(it: Iter<i8>) -> bool {
        correct_size_hint(it.unique())
    }
}

quickcheck! {
    fuzz_group_by_lazy_1(1),
    fn prop(it: Iter<u8>) -> bool {
        let jt = it.clone();
        let groups = it.group_by_lazy(|k| *k);
        let res = itertools::equal(jt, groups.into_iter().flat_map(|(_, x)| x));
        res
    }
}

quickcheck! {
    fuzz_group_by_lazy_2(1),
    fn prop(data: Vec<u8>) -> bool {
        let groups = data.iter().group_by_lazy(|k| *k / 10);
        let res = itertools::equal(data.iter(), groups.into_iter().flat_map(|(_, x)| x));
        res
    }
}

quickcheck! {
    fuzz_group_by_lazy_3(1),
    fn prop(data: Vec<u8>) -> bool {
        let grouper = data.iter().group_by_lazy(|k| *k / 10);
        let groups = grouper.into_iter().collect_vec();
        let res = itertools::equal(data.iter(), groups.into_iter().flat_map(|(_, x)| x));
        res
    }
}

quickcheck! {
    fuzz_group_by_lazy_duo(2),
    fn prop(data: Vec<u8>, order: Vec<(bool, bool)>) -> bool {
        let grouper = data.iter().group_by_lazy(|k| *k / 3);
        let mut groups1 = grouper.into_iter();
        let mut groups2 = grouper.into_iter();
        let mut elts = Vec::<&u8>::new();
        let mut old_groups = Vec::new();

        let tup1 = |(_, b)| b;
        for &(ord, consume_now) in &order {
            let iter = &mut [&mut groups1, &mut groups2][ord as usize];
            match iter.next() {
                Some((_, gr)) => if consume_now {
                    for og in old_groups.drain(..) {
                        elts.extend(og);
                    }
                    elts.extend(gr);
                } else {
                    old_groups.push(gr);
                },
                None => break,
            }
        }
        for og in old_groups.drain(..) {
            elts.extend(og);
        }
        for gr in groups1.map(&tup1) { elts.extend(gr); }
        for gr in groups2.map(&tup1) { elts.extend(gr); }
        itertools::assert_equal(&data, elts);
        true
    }
}

quickcheck! {
    equal_chunks_lazy(2),
    fn prop(a: Vec<u8>, mut size: u8) -> bool {
        if size == 0 {
            size += 1;
        }
        let chunks = a.iter().chunks_lazy(size as usize);
        let it = a.chunks(size as usize);
        for (a, b) in chunks.into_iter().zip(it) {
            if !itertools::equal(a, b) {
                return false;
            }
        }
        true
    }
}

quickcheck! {
    equal_zipslices(2),
    fn prop(a: Vec<u8>, b: Vec<u8>) -> bool {
        use itertools::ZipSlices;
        itertools::equal(ZipSlices::new(&a, &b), a.iter().zip(&b))
    }
}

quickcheck! {
    equal_zipslices_rev(2),
    fn prop(a: Vec<u8>, b: Vec<u8>) -> bool {
        use itertools::ZipSlices;
        itertools::equal(ZipSlices::new(&a, &b).rev(), a.iter().zip(&b).rev())
    }
}

quickcheck! {
    exact_size_zipslices(2),
    fn prop(a: Vec<u8>, b: Vec<u8>) -> bool {
        use itertools::ZipSlices;
        exact_size(ZipSlices::new(&a, &b))
    }
}

quickcheck! {
    exact_size_zipslices_rev(2),
    fn prop(a: Vec<u8>, b: Vec<u8>) -> bool {
        use itertools::ZipSlices;
        exact_size(ZipSlices::new(&a, &b).rev())
    }
}

quickcheck! {
    equal_zipslices_stride(4),
    fn prop(a: Vec<u8>, b: Vec<u8>, mut s1: i8, mut s2: i8) -> bool {
        use itertools::ZipSlices;
        use itertools::Stride;
        if s1 == 0 { s1 += 1; }
        if s2 == 0 { s2 += 1; }
        let a = Stride::from_slice(&a, s1 as isize);
        let b = Stride::from_slice(&b, s2 as isize);
        itertools::equal(ZipSlices::from_slices(a, b), a.zip(b))
    }
}

quickcheck! {
    exact_size_zipslices_stride(4),
    fn prop(a: Vec<u8>, b: Vec<u8>, mut s1: i8, mut s2: i8) -> bool {
        use itertools::ZipSlices;
        use itertools::Stride;
        if s1 == 0 { s1 += 1; }
        if s2 == 0 { s2 += 1; }
        exact_size(ZipSlices::from_slices(Stride::from_slice(&a, s1 as isize),
                                          Stride::from_slice(&b, s2 as isize)))
    }
}
