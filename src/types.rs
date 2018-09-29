use std;
use parinfer;
use serde_json;

pub type LineNumber = usize;
pub type Column = usize;
pub type Delta = i64;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Change {
    pub x: Column,
    pub line_no: LineNumber,
    pub old_text: String,
    pub new_text: String,
}

#[derive(Clone, Deserialize)]
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
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub mode: String,
    pub text: String,
    pub options: Options,
}

#[derive(Clone,Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TabStop<'a> {
    pub ch: &'a str,
    pub x: Column,
    pub line_no: LineNumber,
    pub arg_x: Option<Column>,
}

#[derive(Clone,Debug,Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParenTrail {
    pub line_no: LineNumber,
    pub start_x: Column,
    pub end_x: Column,
}

#[derive(Clone, Debug)]
pub struct Closer<'a> {
    pub line_no: LineNumber,
    pub x: Column,
    pub ch: &'a str,
    pub trail: Option<ParenTrail>
}

#[derive(Clone,Debug,Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Paren<'a> {
    pub line_no: LineNumber,
    pub ch: &'a str,
    pub x: Column,
    pub indent_delta: Delta,
    pub max_child_indent: Option<Column>,
    pub arg_x: Option<Column>,
    pub input_line_no: LineNumber,
    pub input_x: Column,

    #[serde(skip)]
    pub closer: Option<Closer<'a>>,
    #[serde(skip)]
    pub children: Vec<Paren<'a>>
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

