use std;
use parinfer;
use serde_json;

pub type LineNumber = usize;
pub type Column = usize;
pub type Delta = i64;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Change {
    x: Column,
    line_no: LineNumber,
    old_text: String,
    new_text: String,
}

impl Change {
    pub fn to_parinfer(&self) -> parinfer::Change {
        parinfer::Change {
            x: self.x,
            line_no: self.line_no,
            old_text: &self.old_text,
            new_text: &self.new_text,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Options {
    pub cursor_x: Option<Column>,
    pub cursor_line: Option<LineNumber>,
    pub prev_cursor_x: Option<Column>,
    pub prev_cursor_line: Option<LineNumber>,
    pub prev_text: Option<String>,
    pub selection_start_line: Option<LineNumber>,
    #[serde(default = "Options::default_changes")]
    pub changes: Vec<Change>,
    #[serde(default = "Options::default_false")]
    pub partial_result: bool,
    #[serde(default = "Options::default_false")]
    pub force_balance: bool,
    #[serde(default = "Options::default_false")]
    pub return_parens: bool,
}

impl Options {
    fn default_changes() -> Vec<Change> {
        vec![]
    }

    fn default_false() -> bool {
        false
    }

    pub fn to_parinfer(&self) -> parinfer::Options {
        parinfer::Options {
            cursor_x: self.cursor_x,
            cursor_line: self.cursor_line,
            prev_cursor_x: self.prev_cursor_x,
            prev_cursor_line: self.prev_cursor_line,
            selection_start_line: self.selection_start_line,
            changes: self.changes.iter().map(Change::to_parinfer).collect(),
            partial_result: self.partial_result,
            force_balance: self.force_balance,
            return_parens: self.return_parens,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub mode: String,
    pub text: String,
    pub options: Options,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TabStop<'a> {
    ch: &'a str,
    x: Column,
    line_no: LineNumber,
    arg_x: Option<Column>,
}

impl<'a> From<parinfer::TabStop<'a>> for TabStop<'a> {
    fn from(tab_stop: parinfer::TabStop<'a>) -> TabStop<'a> {
        TabStop {
            ch: tab_stop.ch,
            x: tab_stop.x,
            line_no: tab_stop.line_no,
            arg_x: tab_stop.arg_x,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParenTrail {
    line_no: LineNumber,
    start_x: Column,
    end_x: Column,
}

impl From<parinfer::ParenTrail> for ParenTrail {
    fn from(trail: parinfer::ParenTrail) -> ParenTrail {
        ParenTrail {
            line_no: trail.line_no,
            start_x: trail.start_x,
            end_x: trail.end_x,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Paren<'a> {
    line_no: LineNumber,
    ch: &'a str,
    x: Column,
    indent_delta: Delta,
    max_child_indent: Option<Column>,
    arg_x: Option<Column>,
    input_line_no: LineNumber,
    input_x: Column,
}

impl<'a> From<parinfer::Paren<'a>> for Paren<'a> {
    fn from(p: parinfer::Paren<'a>) -> Paren<'a> {
        Paren {
            line_no: p.line_no,
            ch: p.ch,
            x: p.x,
            indent_delta: p.indent_delta,
            max_child_indent: p.max_child_indent,
            arg_x: p.arg_x,
            input_line_no: p.input_line_no,
            input_x: p.input_x,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Answer<'a> {
    pub text: std::borrow::Cow<'a, str>,
    pub success: bool,
    pub error: Option<Error>,
    pub cursor_x: Option<Column>,
    pub cursor_line: Option<LineNumber>,
    pub tab_stops: Vec<TabStop<'a>>,
    pub paren_trails: Vec<ParenTrail>,
    pub parens: Vec<Paren<'a>>,
}

impl<'a> From<parinfer::Answer<'a>> for Answer<'a> {
    fn from(answer: parinfer::Answer<'a>) -> Answer<'a> {
        Answer {
            text: answer.text.clone(),
            success: answer.success,
            error: answer.error.map(Error::from),
            cursor_x: answer.cursor_x,
            cursor_line: answer.cursor_line,
            tab_stops: answer
                .tab_stops
                .iter()
                .map(|t| TabStop::from(t.clone()))
                .collect(),
            paren_trails: answer
                .paren_trails
                .iter()
                .map(|t| ParenTrail::from(t.clone()))
                .collect(),
            parens: answer
                .parens
                .iter()
                .map(|t| Paren::from(t.clone()))
                .collect(),
        }
    }
}

impl<'a> From<Error> for Answer<'a> {
    fn from(error: Error) -> Answer<'a> {
        Answer {
            text: std::borrow::Cow::from(""),
            success: false,
            error: Some(error),
            cursor_x: None,
            cursor_line: None,
            tab_stops: vec![],
            paren_trails: vec![],
            parens: vec![],
        }
    }
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Error {
    pub name: String,
    pub message: String,
    pub x: Option<Column>,
    pub line_no: Option<LineNumber>,
    pub input_x: Option<Column>,
    pub input_line_no: Option<LineNumber>,
}

impl From<parinfer::Error> for Error {
    fn from(error: parinfer::Error) -> Error {
        Error {
            name: error.name.to_string(),
            message: String::from(error.message),
            x: Some(error.x),
            line_no: Some(error.line_no),
            input_x: Some(error.input_x),
            input_line_no: Some(error.input_line_no),
        }
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(error: std::str::Utf8Error) -> Error {
        Error {
            name: String::from("utf8-error"),
            message: format!("Error decoding UTF8: {}", error),
            ..Error::default()
        }
    }
}

impl From<std::ffi::NulError> for Error {
    fn from(error: std::ffi::NulError) -> Error {
        Error {
            name: String::from("nul-error"),
            message: format!("{}", error),
            ..Error::default()
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Error {
        Error {
            name: String::from("json-error"),
            message: format!("Error parsing JSON: {}", error),
            ..Error::default()
        }
    }
}

