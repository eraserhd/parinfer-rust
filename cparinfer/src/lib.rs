extern crate parinfer;

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

extern crate libc;

use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::panic;
use libc::c_char;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Change {
    x: parinfer::Column,
    line_no: parinfer::LineNumber,
    old_text: String,
    new_text: String,
}

impl Change {
    fn to_parinfer(&self) -> parinfer::Change {
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
struct Options {
    cursor_x: Option<parinfer::Column>,
    cursor_line: Option<parinfer::LineNumber>,
    prev_cursor_x: Option<parinfer::Column>,
    prev_cursor_line: Option<parinfer::LineNumber>,
    prev_text: Option<String>,
    selection_start_line: Option<parinfer::LineNumber>,
    #[serde(default = "Options::default_changes")]
    changes: Vec<Change>,
    #[serde(default = "Options::default_false")]
    partial_result: bool,
    #[serde(default = "Options::default_false")]
    force_balance: bool,
    #[serde(default = "Options::default_false")]
    return_parens: bool,
}

impl Options {
    fn default_changes() -> Vec<Change> {
        vec![]
    }

    fn default_false() -> bool {
        false
    }

    fn to_parinfer(&self) -> parinfer::Options {
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
struct Request {
    mode: String,
    text: String,
    options: Options,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TabStop<'a> {
    ch: &'a str,
    x: parinfer::Column,
    line_no: parinfer::LineNumber,
    arg_x: Option<parinfer::Column>,
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
struct ParenTrail {
    line_no: parinfer::LineNumber,
    start_x: parinfer::Column,
    end_x: parinfer::Column,
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
struct Paren<'a> {
    line_no: parinfer::LineNumber,
    ch: &'a str,
    x: parinfer::Column,
    indent_delta: parinfer::Delta,
    max_child_indent: Option<parinfer::Column>,
    arg_x: Option<parinfer::Column>,
    input_line_no: parinfer::LineNumber,
    input_x: parinfer::Column,
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
struct Answer<'a> {
    text: Cow<'a, str>,
    success: bool,
    error: Option<Error>,
    cursor_x: Option<parinfer::Column>,
    cursor_line: Option<parinfer::LineNumber>,
    tab_stops: Vec<TabStop<'a>>,
    paren_trails: Vec<ParenTrail>,
    parens: Vec<Paren<'a>>,
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

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct Error {
    name: String,
    message: String,
    x: Option<parinfer::Column>,
    line_no: Option<parinfer::LineNumber>,
    input_x: Option<parinfer::Column>,
    input_line_no: Option<parinfer::LineNumber>,
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

unsafe fn internal_run(json: *const c_char) -> Result<CString, Error> {
    let json_str = CStr::from_ptr(json).to_str()?;
    let request: Request = serde_json::from_str(json_str)?;
    let mut options = request.options.to_parinfer();

    if let Some(ref prev_text) = request.options.prev_text {
        options.changes.clear();
        if let Some(change) = compute_text_change(prev_text, &request.text) {
            options.changes.push(change);
        }
    }

    let answer: parinfer::Answer;
    if request.mode == "paren" {
        answer = parinfer::paren_mode(&request.text, &options);
    } else if request.mode == "indent" {
        answer = parinfer::indent_mode(&request.text, &options);
    } else if request.mode == "smart" {
        answer = parinfer::smart_mode(&request.text, &options);
    } else {
        return Err(Error {
            message: String::from("Bad value specified for `mode`"),
            ..Error::default()
        });
    }

    let response = serde_json::to_string(&Answer::from(answer))?;

    Ok(CString::new(response)?)
}

/// On unix, Vim loads and unloads the library for every call. On Mac, and
/// possibly other unices, each load creates a new tlv key, and there is a
/// maximum number allowed per process.  When we run out, dlopen() aborts
/// our process.
///
/// Here we reference ourselves and throw the handle away to prevent
/// ourselves from being unloaded (and also set RTLD_NODELETE and
/// RTLD_GLOBAL to make extra sure).
#[cfg(all(unix))]
mod reference_hack {
    use std::ptr;
    use std::ffi::CStr;
    use libc::{c_void, dladdr, dlopen};
    use libc::Dl_info;
    use libc::{RTLD_NOLOAD, RTLD_NODELETE, RTLD_GLOBAL};

    static mut INITIALIZED: bool = false;

    pub unsafe fn initialize() {
        if INITIALIZED {
            return;
        }

        let mut info: Dl_info = Dl_info {
            dli_fname: ptr::null(),
            dli_fbase: ptr::null_mut(),
            dli_sname: ptr::null(),
            dli_saddr: ptr::null_mut()
        };
        let initialize_ptr: *const c_void = &initialize as *const _ as *const c_void;
        if dladdr(initialize_ptr, &mut info) == 0 {
            panic!("Could not get parinfer library path.");
        }
        let handle = dlopen(info.dli_fname, RTLD_NOLOAD|RTLD_GLOBAL|RTLD_NODELETE);
        if handle == ptr::null_mut() {
            panic!("Could not reference cparinfer library {:?}.", CStr::from_ptr(info.dli_fname));
        }
        INITIALIZED = true;
    }
}

#[cfg(not(all(unix)))]
mod reference_hack {
    pub fn initialize() {
    }
}

static mut BUFFER: Option<CString> = None;

#[no_mangle]
pub unsafe extern "C" fn run_parinfer(json: *const c_char) -> *const c_char {
    reference_hack::initialize();
    let output = match panic::catch_unwind(|| internal_run(json)) {
        Ok(Ok(cs)) => cs,
        Ok(Err(e)) => {
            let answer = Answer {
                text: Cow::from(""),
                success: false,
                error: Some(e),
                cursor_x: None,
                cursor_line: None,
                tab_stops: vec![],
                paren_trails: vec![],
                parens: vec![],
            };

            let out = serde_json::to_string(&answer).unwrap();

            CString::new(out).unwrap()
        },
        Err(_) => {
            let answer = Answer {
                text: Cow::from(""),
                success: false,
                error: Some(Error {
                    name: String::from("panic"),
                    message: String::from("plugin panicked!"),
                    x: None,
                    line_no: None,
                    input_x: None,
                    input_line_no: None,

                }),
                cursor_x: None,
                cursor_line: None,
                tab_stops: vec![],
                paren_trails: vec![],
                parens: vec![]
            };

            let out = serde_json::to_string(&answer).unwrap();

            CString::new(out).unwrap()
        }
    };

    BUFFER = Some(output);

    BUFFER.as_ref().unwrap().as_ptr()
}

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
