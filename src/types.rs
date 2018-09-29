use std;
use serde;
use serde_json;
use std::fmt;

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

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum ErrorName {
    QuoteDanger,
    EolBackslash,
    UnclosedQuote,
    UnclosedParen,
    UnmatchedCloseParen,
    UnmatchedOpenParen,
    LeadingCloseParen,

    Utf8EncodingError,
    JsonEncodingError,
    Panic,

    Restart,
}

impl Default for ErrorName {
    fn default() -> ErrorName {
        ErrorName::Restart
    }
}

impl ToString for ErrorName {
    fn to_string(&self) -> String {
        String::from(match self {
            &ErrorName::QuoteDanger => "quote-danger",
            &ErrorName::EolBackslash => "eol-backslash",
            &ErrorName::UnclosedQuote => "unclosed-quote",
            &ErrorName::UnclosedParen => "unclosed-paren",
            &ErrorName::UnmatchedCloseParen => "unmatched-close-paren",
            &ErrorName::UnmatchedOpenParen => "unmatched-open-paren",
            &ErrorName::LeadingCloseParen => "leading-close-paren",
            &ErrorName::Utf8EncodingError => "utf8-error",
            &ErrorName::JsonEncodingError => "json-error",
            &ErrorName::Panic => "panic",
            _ => "??",
        })
    }
}

impl serde::Serialize for ErrorName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'a> serde::Deserialize<'a> for ErrorName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'a>
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = ErrorName;

            fn expecting(&self,  formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("error name")
            }

            fn visit_str<E>(self, value: &str) -> Result<ErrorName, E>
            where E: serde::de::Error
            {
                match value {
                    "quote-danger" => Ok(ErrorName::QuoteDanger),
                     "eol-backslash" => Ok(ErrorName::EolBackslash),
                     "unclosed-quote" => Ok(ErrorName::UnclosedQuote),
                     "unclosed-paren" => Ok(ErrorName::UnclosedParen),
                     "unmatched-close-paren" => Ok(ErrorName::UnmatchedCloseParen),
                     "unmatched-open-paren" => Ok(ErrorName::UnmatchedOpenParen),
                     "leading-close-paren" => Ok(ErrorName::LeadingCloseParen),
                     "utf8-error" => Ok(ErrorName::Utf8EncodingError),
                     "json-error" => Ok(ErrorName::JsonEncodingError),
                     "panic" => Ok(ErrorName::Panic),
                    _ => Err(E::custom(format!("unknown error name: {}", value)))
                }
            }
        }

        deserializer.deserialize_string(Visitor)
    }
}

#[derive(Debug)]
pub struct ErrorExtra {
    pub name: ErrorName,
    pub line_no: LineNumber,
    pub x: Column
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Error {
    pub name: ErrorName,
    pub message: String,
    pub x: Column,
    pub line_no: LineNumber,
    pub input_x: Column,
    pub input_line_no: LineNumber,

    #[serde(skip)]
    pub extra: Option<ErrorExtra>
}

impl From<std::str::Utf8Error> for Error {
    fn from(error: std::str::Utf8Error) -> Error {
        Error {
            name: ErrorName::Utf8EncodingError,
            message: format!("Error decoding UTF8: {}", error),
            ..Error::default()
        }
    }
}

impl From<std::ffi::NulError> for Error {
    fn from(error: std::ffi::NulError) -> Error {
        Error {
            name: ErrorName::Panic,
            message: format!("{}", error),
            ..Error::default()
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Error {
        Error {
            name: ErrorName::JsonEncodingError,
            message: format!("Error parsing JSON: {}", error),
            ..Error::default()
        }
    }
}

