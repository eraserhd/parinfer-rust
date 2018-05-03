use super::*;
use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::panic;
use libc::c_char;
use json::*;

unsafe fn internal_run(json: *const c_char) -> Result<CString, Error> {
    let json_str = CStr::from_ptr(json).to_str()?;
    let request: Request = serde_json::from_str(json_str)?;
    let mut options = request.options.to_parinfer();

    if let Some(ref prev_text) = request.options.prev_text {
        options.changes.clear();
        if let Some(change) = changes::compute_text_change(prev_text, &request.text) {
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
    use libc::{c_void, dladdr, dlerror, dlopen};
    use libc::Dl_info;
    use libc::{RTLD_LAZY, RTLD_NOLOAD, RTLD_NODELETE, RTLD_GLOBAL};

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
        let handle = dlopen(info.dli_fname, RTLD_LAZY|RTLD_NOLOAD|RTLD_GLOBAL|RTLD_NODELETE);
        if handle == ptr::null_mut() {
            panic!("Could not reference cparinfer library {:?}: {:?}.",
                   CStr::from_ptr(info.dli_fname),
                   CStr::from_ptr(dlerror()));
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

#[cfg(not(target_arch = "wasm32"))]
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
