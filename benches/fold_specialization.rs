#![allow(unstable_name_collisions)]

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use itertools::Itertools;

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

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

mod specialization {
    use super::*;

    pub mod intersperse {
        use super::*;

        pub fn external(c: &mut Criterion) {
            let arr = [1; 1024];

            c.bench_function("external", move |b| {
                b.iter(|| {
                    let mut sum = 0;
                    for &x in arr.iter().intersperse(&0) {
                        sum += x;
                    }
                    sum
                })
            });
        }

        pub fn internal_specialized(c: &mut Criterion) {
            let arr = [1; 1024];

            c.bench_function("internal specialized", move |b| {
                b.iter(|| {
                    #[allow(clippy::unnecessary_fold)]
                    arr.iter().intersperse(&0).fold(0, |acc, x| acc + x)
                })
            });
        }

        pub fn internal_unspecialized(c: &mut Criterion) {
            let arr = [1; 1024];

            c.bench_function("internal unspecialized", move |b| {
                b.iter(|| {
                    #[allow(clippy::unnecessary_fold)]
                    Unspecialized(arr.iter().intersperse(&0)).fold(0, |acc, x| acc + x)
                })
            });
        }
    }
}

fn tuple_windows_fold(c: &mut Criterion) {
    let input = (0_u64..4096).collect_vec();
    let mut group = c.benchmark_group("tuple_windows_fold");

    macro_rules! bench_width {
        ($width:expr, $first:tt $(, $rest:tt)*) => {
            group.bench_with_input(
                BenchmarkId::new("specialized", $width),
                &input,
                |b, input| {
                    b.iter(|| {
                        black_box(input)
                            .iter()
                            .copied()
                            .tuple_windows::<(u64, $(bench_width!(@type $rest),)*)>()
                            .fold(0_u64, |acc, window| acc.wrapping_add(window.$first $(^ window.$rest)*))
                    });
                },
            );
            group.bench_with_input(
                BenchmarkId::new("unspecialized", $width),
                &input,
                |b, input| {
                    b.iter(|| {
                        Unspecialized(
                            black_box(input)
                                .iter()
                                .copied()
                                .tuple_windows::<(u64, $(bench_width!(@type $rest),)*)>(),
                        )
                        .fold(0_u64, |acc, window| acc.wrapping_add(window.$first $(^ window.$rest)*))
                    });
                },
            );
        };
        (@type $index:tt) => { u64 };
    }

    bench_width!(1, 0);
    bench_width!(2, 0, 1);
    bench_width!(3, 0, 1, 2);
    bench_width!(4, 0, 1, 2, 3);
    group.finish();
}

fn tuple_windows_chained_fold(c: &mut Criterion) {
    let input = (0_u64..4096).collect_vec();
    let mut group = c.benchmark_group("tuple_windows_chained_fold");

    group.bench_function("specialized", |b| {
        b.iter(|| {
            let (left, right) = black_box(&input).split_at(input.len() / 2);
            left.iter()
                .chain(right)
                .copied()
                .tuple_windows::<(_, _)>()
                .fold(0_u64, |acc, (x, y)| acc.wrapping_add(x ^ y))
        });
    });
    group.bench_function("unspecialized", |b| {
        b.iter(|| {
            let (left, right) = black_box(&input).split_at(input.len() / 2);
            Unspecialized(left.iter().chain(right).copied().tuple_windows::<(_, _)>())
                .fold(0_u64, |acc, (x, y)| acc.wrapping_add(x ^ y))
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    specialization::intersperse::external,
    specialization::intersperse::internal_specialized,
    specialization::intersperse::internal_unspecialized,
    tuple_windows_fold,
    tuple_windows_chained_fold,
);
criterion_main!(benches);
