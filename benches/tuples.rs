#![feature(test)]

extern crate test;
extern crate itertools;

use test::{black_box, Bencher};
use itertools::Itertools;
use itertools::tuples::TupleCollect;

macro_rules! def_benchs {
    ($N:expr; $A:ident, $B:ident, $C:ident, $D:ident, $E:ident, $F:ident; $T:ty) => (
        #[bench]
        fn $A(b: &mut Bencher) {
            let v: Vec<u32> = (0.. $N * 10_000).collect();
            b.iter(|| {
                for x in v.iter().tuples::<$T>() {
                    black_box(&x);
                }
            });
        }

        #[bench]
        fn $B(b: &mut Bencher) {
            let v: Vec<u32> = (0.. $N * 10_000).collect();
            b.iter(|| {
                for x in v.chunks($N) {
                    black_box(&x);
                }
            });
        }

        #[bench]
        fn $C(b: &mut Bencher) {
            let v: Vec<u32> = (0.. $N * 10_000).collect();
            b.iter(|| {
                for x in v.chunks($N) {
                    // create a tuple from the slice
                    let x = <$T>::collect_from_iter_(x);
                    black_box(&x);
                }
            });
        }

        #[bench]
        fn $D(b: &mut Bencher) {
            let v: Vec<u32> = (0..10_000).collect();
            b.iter(|| {
                let mut s = 0;
                for x in v.iter().tuple_windows::<$T>() {
                    s += *x.0;
                }
                s
            });
        }

        #[bench]
        fn $E(b: &mut Bencher) {
            let v: Vec<u32> = (0..10_000).collect();
            b.iter(|| {
                let mut s = 0;
                for x in v.windows($N) {
                    s += x[0];
                }
                s
            });
        }

        #[bench]
        fn $F(b: &mut Bencher) {
            let v: Vec<u32> = (0..10_000).collect();
            b.iter(|| {
                for x in v.windows($N) {
                    // create a tuple from the slice
                    let x = <$T>::collect_from_iter_(x);
                    black_box(&x);
                }
            });
        }
    )
}

def_benchs!{
    1;
    chunks_tuple_1,
    chunks_slice_1,
    chunks_slice_tuple_1,
    windows_tuple_1,
    windows_slice_1,
    windows_slice_tuple_1;
    (&u32, )
}

def_benchs!{
    2;
    chunks_tuple_2,
    chunks_slice_2,
    chunks_slice_tuple_2,
    windows_tuple_2,
    windows_slice_2,
    windows_slice_tuple_2;
    (&u32, &u32)
}

def_benchs!{
    3;
    chunks_tuple_3,
    chunks_slice_3,
    chunks_slice_tuple_3,
    windows_tuple_3,
    windows_slice_3,
    windows_slice_tuple_3;
    (&u32, &u32, &u32)
}

def_benchs!{
    4;
    chunks_tuple_4,
    chunks_slice_4,
    chunks_slice_tuple_4,
    windows_tuple_4,
    windows_slice_4,
    windows_slice_tuple_4;
    (&u32, &u32, &u32, &u32)
}
