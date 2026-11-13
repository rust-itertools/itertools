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

fn kmerge_fold(c: &mut Criterion) {
    let cases = [
        ("single", vec![(0_u64..4096).collect_vec()]),
        (
            "skewed",
            vec![
                (0_u64..32).collect_vec(),
                (32_u64..64).collect_vec(),
                (64_u64..4096).collect_vec(),
            ],
        ),
        (
            "balanced",
            (0_u64..4)
                .map(|offset| (offset..4096).step_by(4).collect_vec())
                .collect_vec(),
        ),
    ];
    let mut group = c.benchmark_group("kmerge_fold");
    for (name, sources) in cases {
        group.bench_with_input(
            BenchmarkId::new("specialized", name),
            &sources,
            |b, sources| {
                b.iter(|| {
                    black_box(sources)
                        .iter()
                        .map(|items| items.iter().copied())
                        .kmerge()
                        .fold(0_u64, u64::wrapping_add)
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("unspecialized", name),
            &sources,
            |b, sources| {
                b.iter(|| {
                    Unspecialized(
                        black_box(sources)
                            .iter()
                            .map(|items| items.iter().copied())
                            .kmerge(),
                    )
                    .fold(0_u64, u64::wrapping_add)
                });
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    specialization::intersperse::external,
    specialization::intersperse::internal_specialized,
    specialization::intersperse::internal_unspecialized,
    kmerge_fold,
);
criterion_main!(benches);
