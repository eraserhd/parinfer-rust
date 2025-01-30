use std::{fmt, mem, rc::Rc};

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

#[derive(Clone, Deserialize, Debug)]
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
    #[serde(default = "Options::default_comment")]
    pub comment_char: char,
    #[serde(default = "Options::default_string_delimiters")]
    pub string_delimiters: Vec<String>,
    #[serde(default = "Options::default_false")]
    pub lisp_vline_symbols: bool,
    #[serde(default = "Options::default_false")]
    pub lisp_block_comments: bool,
    #[serde(default = "Options::default_false")]
    pub guile_block_comments: bool,
    #[serde(default = "Options::default_false")]
    pub scheme_sexp_comments: bool,
    #[serde(default = "Options::default_false")]
    pub janet_long_strings: bool,
    #[serde(default = "Options::default_false")]
    pub hy_bracket_strings: bool,
}

impl Options {
    fn default_changes() -> Vec<Change> {
        vec![]
    }

    fn default_false() -> bool {
        false
    }

    fn default_comment() -> char {
        ';'
    }
    fn default_string_delimiters() -> Vec<String> {
        vec!["\"".to_string()]
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub mode: String,
    pub text: String,
    pub options: Options,
}

#[derive(Clone, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TabStop<'a> {
    pub ch: &'a str,
    pub x: Column,
    pub line_no: LineNumber,
    pub arg_x: Option<Column>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParenTrail {
    pub line_no: LineNumber,
    pub start_x: Column,
    pub end_x: Column,
}

#[derive(Clone, Debug)]
pub struct Closer {
    pub trail: Option<ParenTrail>,
}

#[derive(Clone, Debug, Serialize)]
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
    pub closer: Option<Closer>,
    #[serde(skip)]
    pub children: Vec<Paren<'a>>,
}

#[derive(Serialize, Debug, Clone)]
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

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Default)]
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

    #[default]
    Restart,
}

impl fmt::Display for ErrorName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorName::QuoteDanger => f.write_str("quote-danger"),
            ErrorName::EolBackslash => f.write_str("eol-backslash"),
            ErrorName::UnclosedQuote => f.write_str("unclosed-quote"),
            ErrorName::UnclosedParen => f.write_str("unclosed-paren"),
            ErrorName::UnmatchedCloseParen => f.write_str("unmatched-close-paren"),
            ErrorName::UnmatchedOpenParen => f.write_str("unmatched-open-paren"),
            ErrorName::LeadingCloseParen => f.write_str("leading-close-paren"),
            ErrorName::Utf8EncodingError => f.write_str("utf8-error"),
            ErrorName::JsonEncodingError => f.write_str("json-error"),
            ErrorName::Panic => f.write_str("panic"),
            _ => f.write_str("??"),
        }
    }
}

impl serde::Serialize for ErrorName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'a> serde::Deserialize<'a> for ErrorName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        struct Visitor;

        impl serde::de::Visitor<'_> for Visitor {
            type Value = ErrorName;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("error name")
            }

            fn visit_str<E>(self, value: &str) -> Result<ErrorName, E>
            where
                E: serde::de::Error,
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
                    _ => Err(E::custom(format!("unknown error name: {}", value))),
                }
            }
        }

        deserializer.deserialize_string(Visitor)
    }
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Error {
    pub name: ErrorName,
    pub message: String,
    pub x: Column,
    pub line_no: LineNumber,
    pub input_x: Column,
    pub input_line_no: LineNumber,
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

// Introduce the concept of Reference Counting of requests to work with emacs memory module
#[allow(dead_code)]
pub type SharedRequest = Rc<Request>;

// Info needed to store a pointer to answer
const ANSWER_LEN: usize = mem::size_of::<Answer>() / 8;
pub type RawAnswer = [u64; ANSWER_LEN];

#[allow(dead_code)]
pub struct WrappedAnswer {
    request: SharedRequest,
    raw: RawAnswer,
}

impl WrappedAnswer {
    #[inline]
    #[allow(dead_code)]
    pub unsafe fn new(request: SharedRequest, inner: Answer) -> Self {
        let ptr = (&inner as *const Answer) as *const RawAnswer;
        // Delay inner cursor's cleanup (until wrapper is dropped).
        mem::forget(inner);
        let raw = ptr.read();
        Self { request, raw }
    }

    #[inline]
    #[allow(dead_code)]
    pub fn inner(&self) -> &Answer {
        let ptr = (&self.raw as *const RawAnswer) as *const Answer;
        unsafe { &*ptr }
    }
}

impl Drop for WrappedAnswer {
    #[inline]
    fn drop(&mut self) {
        let ptr = (&mut self.raw as *mut RawAnswer) as *mut Answer;
        unsafe { ptr.read() };
    }
}
