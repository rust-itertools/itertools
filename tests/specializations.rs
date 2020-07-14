use itertools::Itertools;
use std::fmt::Debug;
use quickcheck::quickcheck;

struct Unspecialized<I>(I);
impl<I> Iterator for Unspecialized<I>
where
    I: Iterator,
{
    type Item = I::Item;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

fn check_specialized<'a, V, IterItem, Iter, F>(iterator: &Iter, mapper: F)
where
    V: Eq + Debug,
    Iter: Iterator<Item = IterItem> + Clone + 'a,
    F: Fn(Box<dyn Iterator<Item = IterItem> + 'a>) -> V,
{
    assert_eq!(
        mapper(Box::new(Unspecialized(iterator.clone()))),
        mapper(Box::new(iterator.clone()))
    )
}

fn test_specializations<IterItem, Iter>(
    it: &Iter,
    known_expected_size: Option<usize>,
) where
    IterItem: Eq + Debug + Clone,
    Iter: Iterator<Item = IterItem> + Clone,
{
    let size = it.clone().count();
    if let Some(expected_size) = known_expected_size {
        assert_eq!(size, expected_size);
    }
    check_specialized(it, |i| i.count());
    check_specialized(it, |i| i.last());
    for n in 0..size + 2 {
        check_specialized(it, |mut i| i.nth(n));
    }
    let mut it_sh = it.clone();
    for n in 0..size + 2 {
        let len = it_sh.clone().count();
        let (min, max) = it_sh.size_hint();
        assert_eq!((size - n.min(size)), len);
        assert!(min <= len);
        if let Some(max) = max {
            assert!(len <= max);
        }
        it_sh.next();
    }
    check_specialized(it, |i| {
        let mut parameters_from_fold = vec![];
        let fold_result = i.fold(vec![], |mut acc, v: IterItem| {
            parameters_from_fold.push((acc.clone(), v.clone()));
            acc.push(v);
            acc
        });
        (parameters_from_fold, fold_result)
    });
}

fn put_back_test(test_vec: Vec<i32>, known_expected_size: Option<usize>) {
    {
        // Lexical lifetimes support
        let pb = itertools::put_back(test_vec.iter());
        test_specializations(&pb, known_expected_size);
    }

    let mut pb = itertools::put_back(test_vec.into_iter());
    pb.put_back(1);
    test_specializations(&pb, known_expected_size.map(|x| x + 1));
}

#[test]
fn put_back() {
    put_back_test(vec![7, 4, 1], Some(3));
}

quickcheck! {
    fn put_back_qc(test_vec: Vec<i32>) -> () {
        put_back_test(test_vec, None)
    }
}

fn merge_join_by_test(i1: Vec<usize>, i2: Vec<usize>, known_expected_size: Option<usize>) {
    let i1 = i1.into_iter();
    let i2 = i2.into_iter();
    let mjb = i1.clone().merge_join_by(i2.clone(), std::cmp::Ord::cmp);
    test_specializations(&mjb, known_expected_size);

    // And the other way around
    let mjb = i2.merge_join_by(i1, std::cmp::Ord::cmp);
    test_specializations(&mjb, known_expected_size);
}

#[test]
fn merge_join_by() {
    let i1 = vec![1, 3, 5, 7, 8, 9];
    let i2 = vec![0, 3, 4, 5];
    merge_join_by_test(i1, i2, Some(8));
}

quickcheck! {
    fn merge_join_by_qc(i1: Vec<usize>, i2: Vec<usize>) -> () {
        merge_join_by_test(i1, i2, None)
    }
}
