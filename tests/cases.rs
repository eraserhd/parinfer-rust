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

impl Case {
    fn check<'a>(&self, answer: parinfer::Answer<'a>) {
        assert_eq!(self.result.success, answer.success,
                   "case {}: success", self.source.line_no);
        assert_eq!(self.result.text, answer.text,
                   "case {}: text", self.source.line_no);

        if let Some(x) = self.result.cursor_x {
            assert_eq!(Some(x), answer.cursor_x,
                       "case {}: cursor_x", self.source.line_no);
        }
        if let Some(line_no) = self.result.cursor_line {
            assert_eq!(Some(line_no), answer.cursor_line,
                       "case {}: cursor_line", self.source.line_no);
        }

        if let Some(ref expected) = self.result.error {
            assert!(answer.error.is_some(), "case {}: no error returned");
            let actual = answer.error.unwrap();
            assert_eq!(expected.x, actual.x,
                       "case {}: error.x", self.source.line_no);
            assert_eq!(expected.line_no, actual.line_no,
                       "case {}: error.line_no", self.source.line_no);
            assert_eq!(expected.name, error_str(actual.name),
                       "case {}: error.name", self.source.line_no);
        }

        if let Some(ref tab_stops) = self.result.tab_stops {
            assert_eq!(tab_stops.len(), answer.tab_stops.len(),
                       "case {}: tab stop count", self.source.line_no);
            for (expected, actual) in tab_stops.iter().zip(answer.tab_stops.iter()) {
                assert_eq!(expected.ch, actual.ch,
                           "case {}: tab stop ch", self.source.line_no);
                assert_eq!(expected.x, actual.x,
                           "case {}: tab stop x", self.source.line_no);
                assert_eq!(expected.line_no, actual.line_no,
                           "case {}: tab stop line", self.source.line_no);
                assert_eq!(expected.arg_x, actual.arg_x,
                           "case {}: tab stop arg_x", self.source.line_no);
            }
        }
    }
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
struct TabStop {
    ch: String,
    x: parinfer::Column,
    line_no: parinfer::LineNumber,
    arg_x: Option<parinfer::Column>
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CaseResult {
    text: String,
    success: bool,
    error: Option<Error>,
    cursor_x: Option<parinfer::Column>,
    cursor_line: Option<parinfer::LineNumber>,
    tab_stops: Option<Vec<TabStop>>,
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
        case.check(answer);
    }
}

#[test]
pub fn paren_mode() {
    let cases : Vec<Case> = serde_json::from_str(PAREN_MODE_CASES).unwrap();
    for case in cases {
        let options = case.options.to_parinfer();
        let answer = parinfer::paren_mode(&case.text, &options);
        case.check(answer);
    }
}
