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
    success: bool,
    error: Option<Error>,
    cursor_x: Option<parinfer::Column>,
    cursor_line: Option<parinfer::LineNumber>
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Error {
    name: String,
    line_no: parinfer::LineNumber,
    x: parinfer::Column
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Source {
    line_no: parinfer::LineNumber
}

fn error_str(name: parinfer::ErrorName) -> &'static str {
    match name {
        parinfer::ErrorName::QuoteDanger => "quote-danger",
        parinfer::ErrorName::EolBackslash => "eol-backslash",
        parinfer::ErrorName::UnclosedQuote => "unclosed-quote",
        parinfer::ErrorName::UnmatchedCloseParen => "unmatched-close-paren",
        parinfer::ErrorName::UnmatchedOpenParen => "unmatched-open-paren",
        parinfer::ErrorName::LeadingCloseParen => "leading-close-paren",

        _ => "??"
    }
}

#[test]
pub fn indent_mode() {
    let cases : Vec<Case> = serde_json::from_str(INDENT_MODE_CASES).unwrap();
    for case in cases {
        let options = case.options.to_parinfer();
        let answer = parinfer::indent_mode(&case.text, &options);

        assert_eq!(case.result.success, answer.success,
                   "case {}: success", case.source.line_no);
        assert_eq!(case.result.text, answer.text,
                   "case {}: text", case.source.line_no);

        if let Some(x) = case.result.cursor_x {
            assert_eq!(Some(x), answer.cursor_x,
                       "case {}: cursor_x", case.source.line_no);
        }
        if let Some(line_no) = case.result.cursor_line {
            assert_eq!(Some(line_no), answer.cursor_line,
                       "case {}: cursor_line", case.source.line_no);
        }

        if let Some(expected) = case.result.error {
            assert!(answer.error.is_some(), "case {}: no error returned");
            let actual = answer.error.unwrap();
            assert_eq!(expected.x, actual.x,
                       "case {}: error.x", case.source.line_no);
            assert_eq!(expected.line_no, actual.line_no,
                       "case {}: error.line_no", case.source.line_no);
            assert_eq!(expected.name, error_str(actual.name),
                       "case {}: error.name", case.source.line_no);
        }
    }
}
