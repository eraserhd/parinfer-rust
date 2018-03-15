#![cfg(feature = "unstable")]
#![feature(test)]

extern crate test;
extern crate parinfer;

use test::Bencher;

const LONG_MAP_WITH_STRINGS : &str = include_str!("perf/long_map_with_strings");
const REALLY_LONG_FILE : &str = include_str!("perf/really_long_file");
const REALLY_LONG_FILE_WITH_UNCLOSED_PAREN : &str = include_str!("perf/really_long_file_with_unclosed_paren");
const REALLY_LONG_FILE_WITH_UNCLOSED_QUOTE : &str = include_str!("perf/really_long_file_with_unclosed_quote");

fn default_options<'a>() -> parinfer::Options<'a> {
    parinfer::Options {
        cursor_line: None,
        cursor_x: None,
        changes: vec![],
        force_balance: false,
        partial_result: false,
        prev_cursor_x: None,
        prev_cursor_line: None,
        return_parens: false,
        selection_start_line: None
    }
}

#[bench]
fn bench_paren_long_map_with_strings(b: &mut Bencher) {
    let options = default_options();
    b.iter(|| parinfer::paren_mode(LONG_MAP_WITH_STRINGS, &options));
}

#[bench]
fn bench_indent_long_map_with_strings(b: &mut Bencher) {
    let options = default_options();
    b.iter(|| parinfer::indent_mode(LONG_MAP_WITH_STRINGS, &options));
}

#[bench]
fn bench_smart_long_map_with_strings(b: &mut Bencher) {
   let options = default_options();
   b.iter(|| parinfer::smart_mode(LONG_MAP_WITH_STRINGS, &options));
}

#[bench]
fn bench_paren_really_long_file(b: &mut Bencher) {
    let options = default_options();
    b.iter(|| parinfer::paren_mode(REALLY_LONG_FILE, &options));
}

#[bench]
fn bench_indent_really_long_file(b: &mut Bencher) {
    let options = default_options();
    b.iter(|| parinfer::indent_mode(REALLY_LONG_FILE, &options));
}

#[bench]
fn bench_smart_really_long_file(b: &mut Bencher) {
   let options = default_options();
   b.iter(|| parinfer::smart_mode(REALLY_LONG_FILE, &options));
}

#[bench]
fn bench_paren_really_long_file_with_unclosed_paren(b: &mut Bencher) {
    let options = default_options();
    b.iter(|| parinfer::paren_mode(REALLY_LONG_FILE_WITH_UNCLOSED_PAREN, &options));
}

#[bench]
fn bench_indent_really_long_file_with_unclosed_paren(b: &mut Bencher) {
    let options = default_options();
    b.iter(|| parinfer::indent_mode(REALLY_LONG_FILE_WITH_UNCLOSED_PAREN, &options));
}

#[bench]
fn bench_smart_really_long_file_with_unclosed_paren(b: &mut Bencher) {
   let options = default_options();
   b.iter(|| parinfer::smart_mode(REALLY_LONG_FILE_WITH_UNCLOSED_PAREN, &options));
}

#[bench]
fn bench_quote_really_long_file_with_unclosed_quote(b: &mut Bencher) {
    let options = default_options();
    b.iter(|| parinfer::paren_mode(REALLY_LONG_FILE_WITH_UNCLOSED_QUOTE, &options));
}

#[bench]
fn bench_indent_really_long_file_with_unclosed_quote(b: &mut Bencher) {
    let options = default_options();
    b.iter(|| parinfer::indent_mode(REALLY_LONG_FILE_WITH_UNCLOSED_QUOTE, &options));
}

#[bench]
fn bench_smart_really_long_file_with_unclosed_quote(b: &mut Bencher) {
   let options = default_options();
   b.iter(|| parinfer::smart_mode(REALLY_LONG_FILE_WITH_UNCLOSED_QUOTE, &options));
}
