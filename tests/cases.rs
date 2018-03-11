extern crate parinfer;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

const INDENT_MODE_CASES : &'static str = include_str!("cases/indent-mode.json");
const PAREN_MODE_CASES : &'static str = include_str!("cases/paren-mode.json");
const SMART_MODE_CASES : &'static str = include_str!("cases/smart-mode.json");

#[derive(Deserialize)]
struct Case {
    text: String,
    result: CaseResult,
    source: Source,
    options: Options
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Options {
    cursor_x: Option<parinfer::Column>,
    cursor_line: Option<parinfer::LineNumber>
}

impl Options {
    fn to_parinfer(&self) -> parinfer::Options {
        parinfer::Options {
            cursor_x: self.cursor_x,
            cursor_line: self.cursor_line,
            prev_cursor_x: None,
            prev_cursor_line: None,
            selection_start_line: None,
            changes: vec![],
            partial_result: false,
            force_balance: false,
            return_parens: false
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CaseResult {
    text: String,
    success: bool
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Source {
    line_no: parinfer::LineNumber
}

#[test]
pub fn indent_mode() {
    let cases : Vec<Case> = serde_json::from_str(INDENT_MODE_CASES).unwrap();
    for case in cases {
        let options = case.options.to_parinfer();
        println!("line number: {}", case.source.line_no);
        let answer = parinfer::indent_mode(&case.text, &options);
        if case.result.success {
            assert!(answer.success, "indent_mode() failed when it wasn't supposed to.");
        } else {
            assert!(!answer.success, "indent_mode() succeeded when it wasn't supposed to.");
        }
    }
}
