extern crate parinfer;

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

extern crate libc;

use std::ffi::{CString,CStr};
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
            new_text: &self.new_text
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
    selection_start_line: Option<parinfer::LineNumber>,
    #[serde(default = "Options::default_changes")]
    changes: Vec<Change>,
    #[serde(default = "Options::default_false")]
    partial_result: bool,
    #[serde(default = "Options::default_false")]
    force_balance: bool,
    #[serde(default = "Options::default_false")]
    return_parens: bool
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
            return_parens: self.return_parens
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Request {
    mode: String,
    text: String,
    options: Options
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Answer {
    success: bool,
    error: Option<Error>
}

impl Answer {
    fn from_parinfer<'a>(answer: &parinfer::Answer<'a>) -> Answer {
        Answer {
            success: answer.success,
            error: answer.error.as_ref().map(|e| Error::from_parinfer(e))
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Error {
    message: String
}

impl Error {
    fn from_parinfer(error: &parinfer::Error) -> Error {
        Error {
            message: String::from(error.message)
        }
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(error: std::str::Utf8Error) -> Error {
        Error {
            message: format!("Error decoding UTF8: {}", error)
        }
    }
}

impl From<std::ffi::NulError> for Error {
    fn from(error: std::ffi::NulError) -> Error {
        Error {
            message: format!("{}", error)
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Error {
        Error {
            message: format!("Error parsing JSON: {}", error)
        }
    }
}

static mut BUFFER : *mut c_char = std::ptr::null_mut();

unsafe fn internal_run(json: *const c_char) -> Result<CString, Error> {
    let json_str = CStr::from_ptr(json).to_str()?;
    println!("json_str = {:?}", json_str);
    let request : Request = serde_json::from_str(json_str)?;
    println!("hi!!");
    let options = request.options.to_parinfer();

    let answer : parinfer::Answer;
    if request.mode == "paren" {
        answer = parinfer::paren_mode(&request.text, &options);
    } else if request.mode == "indent" {
        answer = parinfer::indent_mode(&request.text, &options);
    } else if request.mode == "smart" {
        answer = parinfer::smart_mode(&request.text, &options);
    } else {
        //FIXME: Bad mode
        return Err(Error {
            message: String::from("Bad value specified for `mode`")
        });
    }

    let response = serde_json::to_string(&Answer::from_parinfer(&answer))?;

    Ok(CString::new(response)?)
}


#[no_mangle]
pub unsafe extern "C" fn run_parinfer(json: *const c_char) -> *const c_char {

    let output = match internal_run(json) {
        Ok(cs) => cs,
        Err(e) => {
            let answer = Answer {
                success: false,
                error: Some(e)
            };

            let out = serde_json::to_string(&answer).unwrap();

            CString::new(out).unwrap()
        }
    };

    if BUFFER != std::ptr::null_mut() {
        CString::from_raw(BUFFER);
        BUFFER = std::ptr::null_mut();
    }

    BUFFER = output.into_raw();

    BUFFER
}

#[cfg(test)]
mod tests {

    use super::run_parinfer;
    use std::ffi::{CStr,CString};
    use serde_json;
    use serde_json::Value;

    #[test]
    fn it_works() {
        unsafe {
            let json = CString::new("{\"mode\":\"indent\",\"text\":\"(def x\",\"options\":{}}").unwrap();
            let out = CStr::from_ptr(run_parinfer(json.as_ptr())).to_str().unwrap();
            let answer : Value = serde_json::from_str(out).unwrap();
            assert_eq!(Value::Bool(true), answer["success"], "successfully runs parinfer");
        }
    }
}
