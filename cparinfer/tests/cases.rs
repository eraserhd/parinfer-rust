extern crate parinfer;
extern crate cparinfer;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

const INDENT_MODE_CASES: &'static str = include_str!("cases/indent-mode.json");
const PAREN_MODE_CASES: &'static str = include_str!("cases/paren-mode.json");
const SMART_MODE_CASES: &'static str = include_str!("cases/smart-mode.json");

#[derive(Deserialize)]
struct Case {
    text: String,
    result: CaseResult,
    source: Source,
    options: Options,
}

impl Case {
    fn check<'a>(&self, answer: parinfer::Answer<'a>) {
        assert_eq!(
            self.result.success, answer.success,
            "case {}: success",
            self.source.line_no
        );
        assert_eq!(
            self.result.text, answer.text,
            "case {}: text",
            self.source.line_no
        );

        if let Some(x) = self.result.cursor_x {
            assert_eq!(
                Some(x),
                answer.cursor_x,
                "case {}: cursor_x",
                self.source.line_no
            );
        }
        if let Some(line_no) = self.result.cursor_line {
            assert_eq!(
                Some(line_no),
                answer.cursor_line,
                "case {}: cursor_line",
                self.source.line_no
            );
        }

        if let Some(ref expected) = self.result.error {
            assert!(answer.error.is_some(), "case {}: no error returned");
            let actual = answer.error.unwrap();
            assert_eq!(
                expected.x, actual.x,
                "case {}: error.x",
                self.source.line_no
            );
            assert_eq!(
                expected.line_no, actual.line_no,
                "case {}: error.line_no",
                self.source.line_no
            );
            assert_eq!(
                expected.name,
                actual.name.to_string(),
                "case {}: error.name",
                self.source.line_no
            );
        }

        if let Some(ref tab_stops) = self.result.tab_stops {
            assert_eq!(
                tab_stops.len(),
                answer.tab_stops.len(),
                "case {}: tab stop count",
                self.source.line_no
            );
            for (expected, actual) in tab_stops.iter().zip(answer.tab_stops.iter()) {
                assert_eq!(
                    expected.ch, actual.ch,
                    "case {}: tab stop ch",
                    self.source.line_no
                );
                assert_eq!(
                    expected.x, actual.x,
                    "case {}: tab stop x",
                    self.source.line_no
                );
                assert_eq!(
                    expected.line_no, actual.line_no,
                    "case {}: tab stop line",
                    self.source.line_no
                );
                assert_eq!(
                    expected.arg_x, actual.arg_x,
                    "case {}: tab stop arg_x",
                    self.source.line_no
                );
            }
        }

        if let Some(ref trails) = self.result.paren_trails {
            assert_eq!(
                trails.len(),
                answer.paren_trails.len(),
                "case {}: wrong number of paren trails",
                self.source.line_no
            );
            for (expected, actual) in trails.iter().zip(answer.paren_trails.iter()) {
                assert_eq!(
                    expected.line_no, actual.line_no,
                    "case {}: paren trail line number",
                    self.source.line_no
                );
                assert_eq!(
                    expected.start_x, actual.start_x,
                    "case {}: paren trail start x",
                    self.source.line_no
                );
                assert_eq!(
                    expected.end_x, actual.end_x,
                    "case {}: paren trail end x",
                    self.source.line_no
                );
            }
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Options {
    cursor_x: Option<parinfer::Column>,
    cursor_line: Option<parinfer::LineNumber>,
    changes: Option<Vec<Change>>,
    prev_cursor_x: Option<parinfer::Column>,
    prev_cursor_line: Option<parinfer::LineNumber>,
}

impl Options {
    fn to_parinfer(&self) -> parinfer::Options {
        let changes = match self.changes {
            None => vec![],
            Some(ref changes) => changes.iter().map(Change::to_parinfer).collect(),
        };
        parinfer::Options {
            cursor_x: self.cursor_x,
            cursor_line: self.cursor_line,
            prev_cursor_x: self.prev_cursor_x,
            prev_cursor_line: self.prev_cursor_line,
            selection_start_line: None,
            changes,
            partial_result: false,
            force_balance: false,
            return_parens: false,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Change {
    line_no: parinfer::LineNumber,
    x: parinfer::Column,
    old_text: String,
    new_text: String,
}

impl Change {
    fn to_parinfer(&self) -> parinfer::Change {
        parinfer::Change {
            line_no: self.line_no,
            x: self.x,
            new_text: &self.new_text,
            old_text: &self.old_text,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TabStop {
    ch: String,
    x: parinfer::Column,
    line_no: parinfer::LineNumber,
    arg_x: Option<parinfer::Column>,
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
    paren_trails: Option<Vec<ParenTrail>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ParenTrail {
    line_no: parinfer::LineNumber,
    start_x: parinfer::Column,
    end_x: parinfer::Column,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Error {
    name: String,
    line_no: parinfer::LineNumber,
    x: parinfer::Column,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Source {
    line_no: parinfer::LineNumber,
}

#[test]
pub fn indent_mode() {
    let cases: Vec<Case> = serde_json::from_str(INDENT_MODE_CASES).unwrap();
    for case in cases {
        let options = case.options.to_parinfer();
        let answer = parinfer::indent_mode(&case.text, &options);
        case.check(answer);
    }
}

#[test]
pub fn paren_mode() {
    let cases: Vec<Case> = serde_json::from_str(PAREN_MODE_CASES).unwrap();
    for case in cases {
        let options = case.options.to_parinfer();
        let answer = parinfer::paren_mode(&case.text, &options);
        case.check(answer);
    }
}

#[test]
pub fn smart_mode() {
    let cases: Vec<Case> = serde_json::from_str(SMART_MODE_CASES).unwrap();
    for case in cases {
        let options = case.options.to_parinfer();
        let answer = parinfer::smart_mode(&case.text, &options);
        case.check(answer);
    }
}
