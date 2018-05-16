#![feature(test)]

extern crate test;
extern crate parinfer_rust;

#[macro_use]
extern crate serde_json;

use std::ffi::CString;
use test::Bencher;

const LONG_MAP_WITH_STRINGS : &str = include_str!("perf/long_map_with_strings");
const REALLY_LONG_FILE : &str = include_str!("perf/really_long_file");
const REALLY_LONG_FILE_WITH_UNCLOSED_PAREN : &str = include_str!("perf/really_long_file_with_unclosed_paren");
const REALLY_LONG_FILE_WITH_UNCLOSED_QUOTE : &str = include_str!("perf/really_long_file_with_unclosed_quote");

fn build_case(mode: &str, text: &str) -> CString {
    CString::new(json!({
        "mode": mode,
        "text": text,
        "options": {
            "forceBalance": false,
            "partialResult": false,
            "returnParens": false
        }
    }).to_string()).unwrap()
}

#[bench]
fn bench_paren_long_map_with_strings(b: &mut Bencher) {
    unsafe {
        let options = build_case("paren", LONG_MAP_WITH_STRINGS);
        b.iter(|| parinfer_rust::run_parinfer(options.as_ptr()));
    }
}

#[bench]
fn bench_indent_long_map_with_strings(b: &mut Bencher) {
    unsafe {
        let options = build_case("indent", LONG_MAP_WITH_STRINGS);
        b.iter(|| parinfer_rust::run_parinfer(options.as_ptr()));
    }
}

#[bench]
fn bench_smart_long_map_with_strings(b: &mut Bencher) {
    unsafe {
        let options = build_case("smart", LONG_MAP_WITH_STRINGS);
        b.iter(|| parinfer_rust::run_parinfer(options.as_ptr()));
    }
}

#[bench]
fn bench_paren_really_long_file(b: &mut Bencher) {
    unsafe {
        let options = build_case("paren", REALLY_LONG_FILE);
        b.iter(|| parinfer_rust::run_parinfer(options.as_ptr()));
    }
}

#[bench]
fn bench_indent_really_long_file(b: &mut Bencher) {
    unsafe {
        let options = build_case("indent", REALLY_LONG_FILE);
        b.iter(|| parinfer_rust::run_parinfer(options.as_ptr()));
    }
}

#[bench]
fn bench_smart_really_long_file(b: &mut Bencher) {
    unsafe {
        let options = build_case("smart", REALLY_LONG_FILE);
        b.iter(|| parinfer_rust::run_parinfer(options.as_ptr()));
    }
}

#[bench]
fn bench_paren_really_long_file_with_unclosed_paren(b: &mut Bencher) {
    unsafe {
        let options = build_case("paren", REALLY_LONG_FILE_WITH_UNCLOSED_PAREN);
        b.iter(|| parinfer_rust::run_parinfer(options.as_ptr()));
    }
}

#[bench]
fn bench_indent_really_long_file_with_unclosed_paren(b: &mut Bencher) {
    unsafe {
        let options = build_case("indent", REALLY_LONG_FILE_WITH_UNCLOSED_PAREN);
        b.iter(|| parinfer_rust::run_parinfer(options.as_ptr()));
    }
}

#[bench]
fn bench_smart_really_long_file_with_unclosed_paren(b: &mut Bencher) {
    unsafe {
        let options = build_case("smart", REALLY_LONG_FILE_WITH_UNCLOSED_PAREN);
        b.iter(|| parinfer_rust::run_parinfer(options.as_ptr()));
    }
}

#[bench]
fn bench_quote_really_long_file_with_unclosed_quote(b: &mut Bencher) {
    unsafe {
        let options = build_case("paren", REALLY_LONG_FILE_WITH_UNCLOSED_QUOTE);
        b.iter(|| parinfer_rust::run_parinfer(options.as_ptr()));
    }
}

#[bench]
fn bench_indent_really_long_file_with_unclosed_quote(b: &mut Bencher) {
    unsafe {
        let options = build_case("indent", REALLY_LONG_FILE_WITH_UNCLOSED_QUOTE);
        b.iter(|| parinfer_rust::run_parinfer(options.as_ptr()));
    }
}

#[bench]
fn bench_smart_really_long_file_with_unclosed_quote(b: &mut Bencher) {
    unsafe {
        let options = build_case("smart", REALLY_LONG_FILE_WITH_UNCLOSED_QUOTE);
        b.iter(|| parinfer_rust::run_parinfer(options.as_ptr()));
    }
}
