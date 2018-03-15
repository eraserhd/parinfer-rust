#![cfg(feature = "unstable")]
#![feature(test)]

extern crate test;

use test::Bencher;

const LONG_MAP_WITH_STRINGS : &str = include_str!("perf/long_map_with_strings");

#[bench]
fn bench_long_map_with_strings(b: &mut Bencher) {
    b.iter(|| 2 + 2);
}
