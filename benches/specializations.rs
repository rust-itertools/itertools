use criterion::black_box;
use itertools::iproduct;
use itertools::Itertools;

/// Create multiple functions each defining a benchmark group about iterator methods.
///
/// Each created group has functions with the following ids:
///
/// - `next`, `size_hint`, `count`, `last`, `nth`, `collect`, `fold`
/// - and when marked as `DoubleEndedIterator`: `next_back`, `rfold`
/// - and when marked as `ExactSizeIterator`: `len`
///
/// Note that this macro can be called only once.
macro_rules! bench_specializations {
    (
        $(
            $name:ident {
                $($extra:ident)*
                {$(
                    $init:stmt;
                )*}
                $iterator:expr
            }
        )*
    ) => {
        $(
            fn $name(c: &mut ::criterion::Criterion) {
                let mut bench_group = c.benchmark_group(stringify!($name));
                $(
                    $init
                )*
                let bench_first_its = {
                    let mut bench_idx = 0;
                    [0; 1000].map(|_| {
                        let mut it = $iterator;
                        if bench_idx != 0 {
                            it.nth(bench_idx - 1);
                        }
                        bench_idx += 1;
                        it
                    })
                };
                bench_specializations!(@Iterator bench_group bench_first_its: $iterator);
                $(
                    bench_specializations!(@$extra bench_group bench_first_its: $iterator);
                )*
                bench_group.finish();
            }
        )*

        ::criterion::criterion_group!(benches, $($name, )*);
        ::criterion::criterion_main!(benches);
    };

    (@Iterator $group:ident $first_its:ident: $iterator:expr) => {
        $group.bench_function("next", |bencher| bencher.iter(|| {
            let mut it = $iterator;
            while let Some(x) = it.next() {
                black_box(x);
            }
        }));
        $group.bench_function("size_hint", |bencher| bencher.iter(|| {
            $first_its.iter().for_each(|it| {
                black_box(it.size_hint());
            })
        }));
        $group.bench_function("count", |bencher| bencher.iter(|| {
            $iterator.count()
        }));
        $group.bench_function("last", |bencher| bencher.iter(|| {
            $iterator.last()
        }));
        $group.bench_function("nth", |bencher| bencher.iter(|| {
            for start in 0_usize..10 {
                for n in 0..10 {
                    let mut it = $iterator;
                    if let Some(s) = start.checked_sub(1) {
                        black_box(it.nth(s));
                    }
                    while let Some(x) = it.nth(n) {
                        black_box(x);
                    }
                }
            }
        }));
        $group.bench_function("collect", |bencher| bencher.iter(|| {
            $iterator.collect::<Vec<_>>()
        }));
        $group.bench_function("fold", |bencher| bencher.iter(|| {
            $iterator.fold((), |(), x| {
                black_box(x);
            })
        }));
    };

    (@DoubleEndedIterator $group:ident $_first_its:ident: $iterator:expr) => {
        $group.bench_function("next_back", |bencher| bencher.iter(|| {
            let mut it = $iterator;
            while let Some(x) = it.next_back() {
                black_box(x);
            }
        }));
        $group.bench_function("nth_back", |bencher| bencher.iter(|| {
            for start in 0_usize..10 {
                for n in 0..10 {
                    let mut it = $iterator;
                    if let Some(s) = start.checked_sub(1) {
                        black_box(it.nth_back(s));
                    }
                    while let Some(x) = it.nth_back(n) {
                        black_box(x);
                    }
                }
            }
        }));
        $group.bench_function("rfold", |bencher| bencher.iter(|| {
            $iterator.rfold((), |(), x| {
                black_box(x);
            })
        }));
    };

    (@ExactSizeIterator $group:ident $first_its:ident: $_iterator:expr) => {
        $group.bench_function("len", |bencher| bencher.iter(|| {
            $first_its.iter().for_each(|it| {
                black_box(it.len());
            })
        }));
    };
}

// Example: To bench only `ZipLongest::fold`, you can do
//     cargo bench --bench specializations zip_longest/fold
bench_specializations! {
    cartesian_product {
        {
            let v = black_box(vec![0; 16]);
        }
        iproduct!(&v, &v, &v)
    }
    multi_cartesian_product {
        {
            let vs = black_box([0; 3].map(|_| vec![0; 16]));
        }
        vs.iter().multi_cartesian_product()
    }
    tuple_combinations {
        {
            let v = black_box((0..64).collect_vec());
        }
        v.iter().tuple_combinations::<(_, _, _, _)>()
    }
    while_some {
        {}
        (0..)
            .map(black_box)
            .map(|i| char::from_digit(i, 16))
            .while_some()
    }
    with_position {
        ExactSizeIterator
        {
            let v = black_box((0..10240).collect_vec());
        }
        v.iter().with_position()
    }
    zip_longest {
        DoubleEndedIterator
        ExactSizeIterator
        {
            let xs = black_box(vec![0; 1024]);
            let ys = black_box(vec![0; 768]);
        }
        xs.iter().zip_longest(ys.iter())
    }
}
