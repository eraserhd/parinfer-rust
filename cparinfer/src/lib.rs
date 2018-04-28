extern crate parinfer;

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

#[cfg(not(target_arch = "wasm32"))]
extern crate libc;

mod json;
use json::*;

fn compute_text_change<'a>(prev_text: &'a str, text: &'a str) -> Option<parinfer::Change<'a>> {
    let mut x: parinfer::Column = 0;
    let mut line_no: parinfer::LineNumber = 0;
    let mut start_text: usize = 0;
    let mut start_prev: usize = 0;
    let mut end_text: usize = text.len();
    let mut end_prev: usize = prev_text.len();
    let mut different: bool = false;

    for ((i, pc), (j, c)) in prev_text.char_indices().zip(text.char_indices()) {
        if pc != c {
            start_prev = i;
            start_text = j;
            different = true;
            break;
        }
        if pc == '\n' {
            x = 0;
            line_no += 1;
        } else {
            x += 1;
        }
    }

    for ((i, pc), (j, c)) in prev_text.char_indices().rev().zip(text.char_indices().rev()) {
        if pc != c || i < start_prev || j < start_text {
            end_prev = i + pc.len_utf8();
            end_text = j + c.len_utf8();
            break;
        }
    }

    if different {
        Some(parinfer::Change {
            x,
            line_no,
            old_text: &prev_text[start_prev..end_prev],
            new_text: &text[start_text..end_text]
        })
    } else {
        None
    }
}

#[cfg(test)]
#[test]
fn compute_text_change_works() {
    assert_eq!(None, compute_text_change("hello", "hello"));
    assert_eq!(Some(parinfer::Change {
        x: 2,
        line_no: 0,
        old_text: "l",
        new_text: "x"
    }), compute_text_change("hello", "hexlo"));
    assert_eq!(Some(parinfer::Change {
        x: 0,
        line_no: 1,
        old_text: "l",
        new_text: "x"
    }), compute_text_change("he\nllo", "he\nxlo"));
    assert_eq!(Some(parinfer::Change {
        x: 4,
        line_no: 0,
        old_text: "",
        new_text: "l"
    }), compute_text_change("hello", "helllo"));
    assert_eq!(Some(parinfer::Change {
        x: 4,
        line_no: 0,
        old_text: "l",
        new_text: ""
    }), compute_text_change("helllo", "hello"));
}

#[cfg(not(target_arch = "wasm32"))]
mod c_wrapper;

#[cfg(not(target_arch = "wasm32"))]
pub use c_wrapper::run_parinfer;

#[cfg(test)]
mod tests {

    use super::run_parinfer;
    use std::ffi::{CStr, CString};
    use serde_json;
    use serde_json::{Number, Value};

    #[test]
    fn it_works() {
        unsafe {
            let json = CString::new(
                r#"{
                "mode": "indent",
                "text": "(def x",
                "options": {
                    "cursorX": 3,
                    "cursorLine": 0
                }
            }"#,
            ).unwrap();
            let out = CStr::from_ptr(run_parinfer(json.as_ptr()))
                .to_str()
                .unwrap();
            let answer: Value = serde_json::from_str(out).unwrap();
            assert_eq!(
                Value::Bool(true),
                answer["success"],
                "successfully runs parinfer"
            );
            assert_eq!(
                Value::String(String::from("(def x)")),
                answer["text"],
                "returns correct text"
            );
            assert_eq!(
                Value::Number(Number::from(3)),
                answer["cursorX"],
                "returns the correct cursorX"
            );
            assert_eq!(
                Value::Number(Number::from(0)),
                answer["cursorLine"],
                "returns the correct cursorLine"
            );
            assert_eq!(
                Value::Array(vec![]),
                answer["tabStops"],
                "returns the correct tab stops"
            );
            let mut obj: serde_json::map::Map<String, Value> = serde_json::map::Map::new();
            obj.insert(String::from("endX"), Value::Number(Number::from(7)));
            obj.insert(String::from("lineNo"), Value::Number(Number::from(0)));
            obj.insert(String::from("startX"), Value::Number(Number::from(6)));
            assert_eq!(
                Value::Array(vec![Value::Object(obj)]),
                answer["parenTrails"],
                "returns the paren trails"
            );
            assert_eq!(Value::Array(vec![]), answer["parens"], "returns the parens");
        }
    }
}
