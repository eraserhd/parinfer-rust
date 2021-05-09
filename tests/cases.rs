extern crate parinfer_rust;
extern crate serde;
#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

#[cfg(not(target_arch = "wasm32"))]
use std::ffi::{CStr, CString};

const INDENT_MODE_CASES: &'static str = include_str!("cases/indent-mode.json");
const PAREN_MODE_CASES: &'static str = include_str!("cases/paren-mode.json");
const SMART_MODE_CASES: &'static str = include_str!("cases/smart-mode.json");

type LineNumber = usize;
type Column = usize;

#[derive(Deserialize)]
struct Case {
    text: String,
    result: CaseResult,
    source: Source,
    options: Options,
}

impl Case {
    fn check2(&self, answer: serde_json::Value) {
        assert_eq!(
            json!(self.result.success), answer["success"],
            "case {}: success",
            self.source.line_no
        );
        assert_eq!(
            self.result.text, answer["text"],
            "case {}: text",
            self.source.line_no
        );

        if let Some(x) = self.result.cursor_x {
            assert_eq!(
                json!(x),
                answer["cursorX"],
                "case {}: cursor_x",
                self.source.line_no
            );
        }
        if let Some(line_no) = self.result.cursor_line {
            assert_eq!(
                json!(line_no),
                answer["cursorLine"],
                "case {}: cursor_line",
                self.source.line_no
            );
        }

        if let Some(ref expected) = self.result.error {
            assert_eq!(
                json!(expected.x), answer["error"]["x"],
                "case {}: error.x",
                self.source.line_no
            );
            assert_eq!(
                json!(expected.line_no), answer["error"]["lineNo"],
                "case {}: error.line_no",
                self.source.line_no
            );
            assert_eq!(
                json!(expected.name),
                answer["error"]["name"],
                "case {}: error.name",
                self.source.line_no
            );
        }

        if let Some(ref tab_stops) = self.result.tab_stops {
            assert_eq!(
                tab_stops.len(),
                answer["tabStops"].as_array().unwrap().len(),
                "case {}: tab stop count",
                self.source.line_no
            );
            for (expected, actual) in tab_stops.iter().zip(answer["tabStops"].as_array().unwrap().iter()) {
                assert_eq!(
                    json!(expected.ch), actual["ch"],
                    "case {}: tab stop ch",
                    self.source.line_no
                );
                assert_eq!(
                    json!(expected.x), actual["x"],
                    "case {}: tab stop x",
                    self.source.line_no
                );
                assert_eq!(
                    json!(expected.line_no), actual["lineNo"],
                    "case {}: tab stop line",
                    self.source.line_no
                );
                assert_eq!(
                    json!(expected.arg_x), actual["argX"],
                    "case {}: tab stop arg_x",
                    self.source.line_no
                );
            }
        }

        if let Some(ref trails) = self.result.paren_trails {
            assert_eq!(
                trails.len(),
                answer["parenTrails"].as_array().unwrap().len(),
                "case {}: wrong number of paren trails",
                self.source.line_no
            );
            for (expected, actual) in trails.iter().zip(answer["parenTrails"].as_array().unwrap().iter()) {
                assert_eq!(
                    expected.line_no, actual["lineNo"],
                    "case {}: paren trail line number",
                    self.source.line_no
                );
                assert_eq!(
                    expected.start_x, actual["startX"],
                    "case {}: paren trail start x",
                    self.source.line_no
                );
                assert_eq!(
                    expected.end_x, actual["endX"],
                    "case {}: paren trail end x",
                    self.source.line_no
                );
            }
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Options {
    cursor_x: Option<Column>,
    cursor_line: Option<LineNumber>,
    #[serde(skip_serializing_if = "Option::is_none")]
    changes: Option<Vec<Change>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prev_cursor_x: Option<Column>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prev_cursor_line: Option<LineNumber>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lisp_vline_symbols: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lisp_block_comments: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    guile_block_comments: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scheme_sexp_comments: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    janet_long_strings: Option<bool>,
}


#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Change {
    line_no: LineNumber,
    x: Column,
    old_text: String,
    new_text: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TabStop {
    ch: String,
    x: Column,
    line_no: LineNumber,
    arg_x: Option<Column>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CaseResult {
    text: String,
    success: bool,
    error: Option<Error>,
    cursor_x: Option<Column>,
    cursor_line: Option<LineNumber>,
    tab_stops: Option<Vec<TabStop>>,
    paren_trails: Option<Vec<ParenTrail>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ParenTrail {
    line_no: LineNumber,
    start_x: Column,
    end_x: Column,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Error {
    name: String,
    line_no: LineNumber,
    x: Column,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Source {
    line_no: LineNumber,
}

#[cfg(not(target_arch = "wasm32"))]
fn run(input: &str) -> String {
    unsafe {
        parinfer_rust::INITIALIZED = true;
        let c_input = CString::new(input).unwrap();
        String::from(CStr::from_ptr(parinfer_rust::run_parinfer(c_input.as_ptr())).to_str().unwrap())
    }
}

#[cfg(target_arch = "wasm32")]
fn run(input: &str) -> String {
    parinfer_rust::run_parinfer(String::from(input))
}

#[test]
pub fn indent_mode() {
    let cases: Vec<Case> = serde_json::from_str(INDENT_MODE_CASES).unwrap();
    for case in cases {
        let input = json!({
            "mode": "indent",
            "text": &case.text,
            "options": &case.options
        }).to_string();
        let answer: serde_json::Value = serde_json::from_str(&run(&input)).unwrap();
        case.check2(answer);
    }
}

#[test]
pub fn paren_mode() {
    let cases: Vec<Case> = serde_json::from_str(PAREN_MODE_CASES).unwrap();
    for case in cases {
        let input = json!({
            "mode": "paren",
            "text": &case.text,
            "options": &case.options
        }).to_string();
        let answer: serde_json::Value = serde_json::from_str(&run(&input)).unwrap();
        case.check2(answer);
    }
}

#[test]
pub fn smart_mode() {
    let cases: Vec<Case> = serde_json::from_str(SMART_MODE_CASES).unwrap();
    for case in cases {
        let input = json!({
            "mode": "smart",
            "text": &case.text,
            "options": &case.options
        }).to_string();
        let answer: serde_json::Value = serde_json::from_str(&run(&input)).unwrap();
        case.check2(answer);
    }
}

#[test]
pub fn composed_unicode_graphemes_count_as_a_single_character() {
    let case = Case {
        text: String::from("(éééé (\nh))"),
        result: CaseResult {
            text: String::from("(éééé (\n       h))"),
            success: true,
            error: None,
            cursor_x: None,
            cursor_line: None,
            tab_stops: None,
            paren_trails: None
        },
        source: Source {
            line_no: 0
        },
        options: Options {
            cursor_x: None,
            cursor_line: None,
            changes: None,
            lisp_vline_symbols: None,
            lisp_block_comments: None,
            guile_block_comments: None,
            scheme_sexp_comments: None,
            janet_long_strings: None,
            prev_cursor_x: None,
            prev_cursor_line: None
        }
    };
    let input = json!({
        "mode": "paren",
        "text": &case.text,
        "options": &case.options
    }).to_string();
    let answer: serde_json::Value = serde_json::from_str(&run(&input)).unwrap();
    case.check2(answer);
}

#[test]
pub fn graphemes_in_changes_are_counted_correctly() {
    let case = Case {
        text: String::from("(éxyåååé [h\n       i])"),
        result: CaseResult {
            text: String::from("(éxyåååé [h\n          i])"),
            success: true,
            error: None,
            cursor_x: Some(10),
            cursor_line: Some(1),
            tab_stops: None,
            paren_trails: None
        },
        source: Source {
            line_no: 0
        },
        options: Options {
            cursor_x: Some(7),
            cursor_line: Some(1),
            changes: Some(vec![
                Change {
                    line_no: 0,
                    x: 2,
                    old_text: String::from("éé"),
                    new_text: String::from("xyååå"),
                }
            ]),
            lisp_vline_symbols: None,
            lisp_block_comments: None,
            guile_block_comments: None,
            scheme_sexp_comments: None,
            janet_long_strings: None,
            prev_cursor_x: None,
            prev_cursor_line: None
        }
    };
    let input = json!({
        "mode": "smart",
        "text": &case.text,
        "options": &case.options
    }).to_string();
    let answer: serde_json::Value = serde_json::from_str(&run(&input)).unwrap();
    case.check2(answer);
}

#[test]
pub fn wide_characters() {
    let case = Case {
        text: String::from("(def ｗｏｒｌｄ {:foo 1\n            :bar 2})"),
        result: CaseResult {
            text: String::from("(def ｗｏｒｌｄ {:foo 1\n                 :bar 2})"),
            success: true,
            error: None,
            cursor_x: Some(17),
            cursor_line: Some(1),
            tab_stops: None,
            paren_trails: None
        },
        source: Source {
            line_no: 0
        },
        options: Options {
            cursor_x: Some(12),
            cursor_line: Some(1),
            changes: Some(vec![
                Change {
                    line_no: 0,
                    x: 5,
                    old_text: String::from("world"),
                    new_text: String::from("ｗｏｒｌｄ"),
                }
            ]),
            lisp_vline_symbols: None,
            lisp_block_comments: None,
            guile_block_comments: None,
            scheme_sexp_comments: None,
            janet_long_strings: None,
            prev_cursor_x: None,
            prev_cursor_line: None
        }
    };
    let input = json!({
        "mode": "smart",
        "text": &case.text,
        "options": &case.options
    }).to_string();
    let answer: serde_json::Value = serde_json::from_str(&run(&input)).unwrap();
    case.check2(answer);
}

#[test]
pub fn lisp_vline_symbols() {
    let case = Case {
        text: String::from("(define foo |Not a closing parenthesis )|)"),
        result: CaseResult {
            text: String::from("(define foo |Not a closing parenthesis )|)"),
            success: true,
            error: None,
            cursor_x: None,
            cursor_line: None,
            tab_stops: None,
            paren_trails: None
        },
        source: Source {
            line_no: 0
        },
        options: Options {
            cursor_x: None,
            cursor_line: None,
            changes: None,
            lisp_vline_symbols: Some(true),
            lisp_block_comments: None,
            guile_block_comments: None,
            scheme_sexp_comments: None,
            janet_long_strings: None,
            prev_cursor_x: None,
            prev_cursor_line: None
        }
    };
    let input = json!({
        "mode": "paren",
        "text": &case.text,
        "options": &case.options
    }).to_string();
    let answer: serde_json::Value = serde_json::from_str(&run(&input)).unwrap();
    case.check2(answer);
}

#[test]
pub fn lisp_sharp_syntax_backtrack() {
    let case = Case {
        text: String::from("(let ((x #(1 2 3))) x)"),
        result: CaseResult {
            text: String::from("(let ((x #(1 2 3))) x)"),
            success: true,
            error: None,
            cursor_x: None,
            cursor_line: None,
            tab_stops: None,
            paren_trails: None
        },
        source: Source {
            line_no: 0
        },
        options: Options {
            cursor_x: None,
            cursor_line: None,
            changes: None,
            lisp_vline_symbols: None,
            lisp_block_comments: Some(true),
            guile_block_comments: None,
            scheme_sexp_comments: None,
            janet_long_strings: None,
            prev_cursor_x: None,
            prev_cursor_line: None
        }
    };
    let input = json!({
        "mode": "paren",
        "text": &case.text,
        "options": &case.options
    }).to_string();
    let answer: serde_json::Value = serde_json::from_str(&run(&input)).unwrap();
    case.check2(answer);
}

#[test]
pub fn lisp_block_comments() {
    let case = Case {
        text: String::from("'(#| this #| is |# nested ) comment |# passed through)"),
        result: CaseResult {
            text: String::from("'(#| this #| is |# nested ) comment |# passed through)"),
            success: true,
            error: None,
            cursor_x: None,
            cursor_line: None,
            tab_stops: None,
            paren_trails: None
        },
        source: Source {
            line_no: 0
        },
        options: Options {
            cursor_x: None,
            cursor_line: None,
            changes: None,
            lisp_vline_symbols: None,
            lisp_block_comments: Some(true),
            guile_block_comments: None,
            scheme_sexp_comments: None,
            janet_long_strings: None,
            prev_cursor_x: None,
            prev_cursor_line: None
        }
    };
    let input = json!({
        "mode": "paren",
        "text": &case.text,
        "options": &case.options
    }).to_string();
    let answer: serde_json::Value = serde_json::from_str(&run(&input)).unwrap();
    case.check2(answer);
}

#[test]
pub fn guile_block_comments() {
    let case = Case {
        text: String::from("#!/bin/guile -s \\\n-e main -s\n!#\n(display\n'hello)"),
        result: CaseResult {
            text: String::from("#!/bin/guile -s \\\n-e main -s\n!#\n(display)\n'hello"),
            success: true,
            error: None,
            cursor_x: None,
            cursor_line: None,
            tab_stops: None,
            paren_trails: None
        },
        source: Source {
            line_no: 0
        },
        options: Options {
            cursor_x: None,
            cursor_line: None,
            changes: None,
            lisp_vline_symbols: None,
            lisp_block_comments: None,
            guile_block_comments: Some(true),
            scheme_sexp_comments: None,
            janet_long_strings: None,
            prev_cursor_x: None,
            prev_cursor_line: None
        }
    };
    let input = json!({
        "mode": "indent",
        "text": &case.text,
        "options": &case.options
    }).to_string();
    let answer: serde_json::Value = serde_json::from_str(&run(&input)).unwrap();
    case.check2(answer);
}

#[test]
pub fn scheme_sexp_comments() {
    let case = Case {
        text: String::from("'(#; (ignored here) not ignored"),
        result: CaseResult {
            text: String::from("'(#; (ignored here) not ignored)"),
            success: true,
            error: None,
            cursor_x: None,
            cursor_line: None,
            tab_stops: None,
            paren_trails: None
        },
        source: Source {
            line_no: 0
        },
        options: Options {
            cursor_x: None,
            cursor_line: None,
            changes: None,
            lisp_vline_symbols: None,
            lisp_block_comments: None,
            guile_block_comments: None,
            scheme_sexp_comments: Some(true),
            janet_long_strings: None,
            prev_cursor_x: None,
            prev_cursor_line: None
        }
    };
    let input = json!({
        "mode": "indent",
        "text": &case.text,
        "options": &case.options
    }).to_string();
    let answer: serde_json::Value = serde_json::from_str(&run(&input)).unwrap();
    case.check2(answer);
}

#[test]
pub fn janet_long_strings() {
    let case = Case {
        text: String::from("(def foo {:bar `Not a closing parenthesis )`})"),
        result: CaseResult {
            text: String::from("(def foo {:bar `Not a closing parenthesis )`})"),
            success: true,
            error: None,
            cursor_x: None,
            cursor_line: None,
            tab_stops: None,
            paren_trails: None
        },
        source: Source {
            line_no: 0
        },
        options: Options {
            cursor_x: None,
            cursor_line: None,
            changes: None,
            lisp_vline_symbols: None,
            lisp_block_comments: None,
            guile_block_comments: None,
            scheme_sexp_comments: None,
            janet_long_strings: Some(true),
            prev_cursor_x: None,
            prev_cursor_line: None
        }
    };
    let input = json!({
        "mode": "paren",
        "text": &case.text,
        "options": &case.options
    }).to_string();
    let answer: serde_json::Value = serde_json::from_str(&run(&input)).unwrap();
    case.check2(answer);
}
