use super::parinfer::rc_process;
use emacs::{Env, IntoLisp, Result, Value};
use types::{Change, Error, Options, Request, SharedRequest, WrappedAnswer};

use std::{fs::OpenOptions,
          io::Write,
          convert::TryFrom,
          cell::RefCell,
          rc::Rc};

emacs::plugin_is_GPL_compatible!();

// This exists because the emacs library doesn't implement
// this type conversion directy
/// A helper function to Option<i64> to Option<usize>
fn to_usize(value: Option<i64>) -> Option<usize> {
  match value {
    Some(item) => match usize::try_from(item) {
      Ok(new_value) => Some(new_value),
      Err(_) => None,
    },
    None => None,
  }
}

fn to_i64(value: Option<usize>) -> Option<i64> {
  match value {
    Some(item) => match i64::try_from(item) {
      Ok(new_value) => Some(new_value),
      Err(_) => None,
    },
    None => None,
  }
}

#[emacs::module(name = "parinfer-rust")]
pub fn init(_: &Env) -> Result<()> {
  Ok(())
}

////////////////////////////////
// Entry point
///////////////////////////////
// Need to wrap Request into a specific lifetime for use as part of this struct are used in the Answer struct
// this is talked about here https://github.com/ubolonton/emacs-module-rs/issues/21
type AliasedRequest<'a> = &'a SharedRequest;

//  https://github.com/shaunlebron/parinfer/tree/master/lib#api
// text is the full text input.
// options is an object with the following properties:
//   cursorLine - zero-based line number of the cursor
//   cursorX - zero-based x-position of the cursor
//   prevCursorLine and prevCursorX is required by Smart Mode (previous cursor position)
//   selectionStartLine - first line of the current selection
//   changes - ordered array of change objects with the following:
//     lineNo - starting line number of the change
//     x - starting x of the change
//     oldText - original text that was replaced
//     newText - new text that replaced the original text
//   forceBalance - employ the aggressive paren-balancing rules from v1 (defaults to false)
//   partialResult - return partially processed text/cursor if an error occurs (defaults to false)

// success is a boolean indicating if the input was properly formatted enough to create a valid result
// text is the full text output (if success is false, returns original text unless partialResult is enabled)
// cursorX/cursorLine is the new position of the cursor (since parinfer may shift it around)
// error is an object populated if success is false:
//   name is the name of the error, which will be any of the following:
//     "quote-danger"
//     "eol-backslash"
//     "unclosed-quote"
//     "unclosed-paren"
//     "unmatched-close-paren"
//     "unhand led"
//   message is a message describing the error
//   lineNo is a zero-based line number where the error occurred
//   x is a zero-based column where the error occurred
//   extra has lineNo and x of open-paren for unmatched-close-paren
// tabStops is an array of objects representing Tab stops, which is populated if a cursor position or selection is supplied. We identify tab stops at relevant open-parens, and supply the following extra information so you may compute extra tab stops for one-space or two-space indentation conventions based on the type of open-paren.
//   x is a zero-based x-position of the tab stop
//   argX position of the first argument after x (e.g. position of bar in (foo bar)
//   lineNo is a zero-based line number of the open-paren responsible for the tab stop
//   ch is the character of the open-paren responsible for the tab stop (e.g. (,[,{)
// parenTrails is an array of object representing the Paren Trails at the end of each line that Parinfer may move
//   lineNo is a zero-based line number
//   startX is a zero-based x-position of the first close-paren
//   endX is a zero-based x-position after the last close-paren
#[defun(user_ptr, mod_in_name = false)]
/// Runs the parinfer algorithm on the given request
///
/// # Examples
///
/// ```elisp,no_run
/// (parinfer-rust-execute request)
/// ```
fn execute(request: AliasedRequest) -> Result<WrappedAnswer> {
  let answer = rc_process(&request);
  let wrapped_answer = unsafe{WrappedAnswer::new(request.clone(), answer)};
  Ok(wrapped_answer)
}
////////////////////////////////
// options
////////////////////////////////
#[defun(user_ptr, mod_in_name = false)]
// Create an Options Structure
// We need this because we can't pass in an optional variant of Options in the new_options function
/// Returns an Option with nil data for all fields
///
/// # Examples
///
/// ```elisp,no_run
/// (parinfer-make-option)
/// ```
fn make_option() -> Result<Options> {
  Ok(Options {
    cursor_x: None,
    cursor_line: None,
    prev_cursor_x: None,
    prev_cursor_line: None,
    prev_text: None,
    selection_start_line: None,
    changes: Vec::new(),
    partial_result: false,
    force_balance: false,
    return_parens: false,
    comment_char: ';',
    string_delimiters: Vec::new(),
    lisp_vline_symbols: false,
    lisp_block_comments: false,
    guile_block_comments: false,
    scheme_sexp_comments: false,
    janet_long_strings: false,
  })
}

#[defun(user_ptr, mod_in_name = false)]
/// Creates an Options type based on inputs
///
/// # Examples
///
/// ```elisp,no_run
/// (parinfer-new-option 1 1 nil options changes)
/// ```
fn new_options(
  cursor_x: Option<i64>,
  cursor_line: Option<i64>,
  selection_start_line: Option<i64>,
  old_options: &Options,
  changes: &Vec<Change>,
) -> Result<Options> {
  Ok(Options {
    cursor_x: to_usize(cursor_x),
    cursor_line: to_usize(cursor_line),
    prev_cursor_x: old_options.cursor_x,
    prev_cursor_line: old_options.cursor_line,
    selection_start_line: to_usize(selection_start_line),
    changes: changes.clone(),
    prev_text: None,
    partial_result: false,
    force_balance: false,
    return_parens: false,
    comment_char: ';',
    string_delimiters: Vec::new(),
    lisp_vline_symbols: false,
    lisp_block_comments: false,
    guile_block_comments: false,
    scheme_sexp_comments: false,
    janet_long_strings: false,
  })
}

#[defun(mod_in_name = false)]
/// Returns a string representation of the Options type
///
/// # Examples
///
/// ```elisp,no_run
/// (parinfer-print-option options)
/// ```
fn print_options<'a>(options: &Options) -> Result<String> {
  Ok(format!("{:?}", options.clone()).to_string())
}

////////////////////////////////
// Changes
////////////////////////////////
#[defun(user_ptr, mod_in_name = false)]

/// Creates an empty list of changes
///
/// # Examples
///
/// ```elisp,no_run
/// (parinfer-make-changes)
/// ```
fn make_changes() -> Result<Vec<Change>> {
  Ok(Vec::new())
}
#[defun(user_ptr, mod_in_name = false)]
fn new_change(line_number: i64, x: i64, old_text: String, new_text: String) -> Result<Change> {
  let line_no = usize::try_from(line_number)?;
  let new_x = usize::try_from(x)?;
  let change = Change {
    x: new_x,
    line_no,
    old_text,
    new_text,
  };
  Ok(change)
}

#[defun(mod_in_name = false)]
/// Creates an empty list of changes
///
/// # Examples
///
/// ```elisp,no_run
/// (parinfer-make-changes)
/// ```
fn add_change(changes: &mut Vec<Change>, change: &Change) -> Result<()> {
  Ok(changes.push(change.clone()))
}

#[defun(mod_in_name = false)]
/// Returns a string representing a list of changes
///
/// # Examples
///
/// ```elisp,no_run
/// (parinfer-print-changes changes)
/// ```
fn print_changes<'a>(env: &'a Env, changes: &mut Vec<Change>) -> Result<Value<'a>> {
  format!("{:?}", changes).into_lisp(env)
}

////////////////////////////////
// Request
////////////////////////////////
#[defun(mod_in_name = false)]

/// Creates a Request from the given mode, current buffer text, and the set of Options
///
/// # Examples
///
/// ```elisp,no_run
/// (parinfer-make-request "paren" (buffer-substring-no-properties) options)
/// ```
//
fn make_request(mode: String, text: String, options: &mut Options) -> Result<SharedRequest> {
  let request = Request {
    mode,
    text,
    options: options.clone(),
  };
  Ok(Rc::new(request))
}

/// Creates a Request from the given mode, current buffer text, and the set of Options
///
/// # Examples
///
/// ```elisp,no_run
/// (parinfer-print-request request)
/// ```
//
#[defun(mod_in_name = false)]
fn print_request(request: AliasedRequest) -> Result<String> {
  Ok(format!("{:?}", &request).to_string())
}
////////////////////////////////
// Answer
////////////////////////////////
#[defun(mod_in_name = false)]
/// Gives a hashmap like interface to extracting values from the Answer type
/// Accepted keys are 'text', 'success', 'cursor_x', 'cursor_line', and 'error'
///
/// # Examples
///
/// ```elisp,no_run
/// (parinfer-get-in-answer answer "success")
/// ```
fn get_in_answer<'a>(
  env: &'a Env,
  answer: &WrappedAnswer,
  key: Option<String>,
) -> Result<Value<'a>> {
  let unwrapped_answer = answer.inner();
  let query = match key {
    Some(key) => key,
    None => return env.message("Missing 'key'"),
  };

  // I only care about some nested structures at the moment, errors,
  // so leave tab_stops, paren_trails, and parens as unsupported
  match query.as_ref() {
    "text" => unwrapped_answer.text.to_string().into_lisp(env),
    "success" => unwrapped_answer.success.into_lisp(env),
    "cursor_x" => to_i64(unwrapped_answer.cursor_x).into_lisp(env),
    "cursor_line" => to_i64(unwrapped_answer.cursor_line).into_lisp(env),
    "error" => match unwrapped_answer.error.clone() {
      Some(error) => Ok(RefCell::new(error).into_lisp(env)?),
      None => ().into_lisp(env),
    }
    // "tab_stops"
    // "paren_trails"
    // "parens"
    _ => {
      env.message(format!("Key '{}' unsupported", query))?;
      ().into_lisp(env)},
  }
}

#[defun(mod_in_name = false)]
/// Returns a string representation of an Answer
///
/// # Examples
///
/// ```elisp,no_run
/// (parinfer-rust-print-answer answer)
/// ```
fn print_answer(answer: &WrappedAnswer) -> Result<String> {
  Ok(format!("{:?}", answer.inner()).to_string())
}

#[defun(mod_in_name = false)]
/// Prints the current Options and Answer to the specified file
///
/// # Examples
///
/// ```elisp,no_run
/// (parinfer-rust-debug "/tmp/parinfer.txt" options answer)
/// ```
fn debug(env: &Env, filename: String, options: &Options, wrapped_answer: &WrappedAnswer) -> Result<()> {
  let answer = wrapped_answer.inner();
  let file = match OpenOptions::new().append(true).create(true).open(&filename) {
    Ok(file) => file,
    Err(_) => {
      env.message(&format!("Unable to open file {}", filename))?;
      return Ok(());
    }
  };

  match write!(&file, "Options:\n{:?}\nResponse:\n{:?}\n", options, answer) {
    Ok(_) => {
      env.message(&format!("Wrote debug information to {}", filename))?;
    }
    Err(_) => {
      env.message(&format!("Unable to write to file {}", filename))?;
    }
  };
  Ok(())
}

////////////////////////////////
// Error
////////////////////////////////
#[defun(mod_in_name = false)]
/// Gives a hashmap like interface to extracting values from the error type
/// Accepted keys are 'text', 'success', 'cursor_x', 'cursor_line', and 'error'
///
/// # Examples
///
/// ```elisp,no_run
/// (parinfer-get-in-error error "message")
/// ```
fn get_in_error<'a>(env: &'a Env, error: &Error, key: Option<String>)-> Result<Value<'a>> {
  let query = match key {
    Some(key) => key,
    None => "".to_string(),
  };

  match query.as_ref() {
    "name" => error.name.to_string().into_lisp(env),
    "message" => error.message.clone().into_lisp(env),
    "x" => to_i64(Some(error.x)).into_lisp(env),
    "line_no" => to_i64(Some(error.line_no)).into_lisp(env),
    "input_x" => to_i64(Some(error.input_x)).into_lisp(env),
    "input_line_no" => to_i64(Some(error.input_line_no)).into_lisp(env),
    _ => {
      env.message(format!("Key '{}' unsupported", query))?; // Can return an error
      ().into_lisp(env)
    }
  }
}

#[defun(mod_in_name = false)]
/// Returns a string representation of an Error
///
/// # Examples
///
/// ```elisp,no_run
/// (parinfer-rust-print-error error)
/// ```
fn print_error(error: &Error) -> Result<String>{
  Ok(format!("{:?}", error).to_string())
}

#[defun(mod_in_name = false)]
/// Returns the version of the parinfer-rust library
///
/// # Examples
///
/// ```elisp,no_run
/// (parinfer-rust-version)
/// ```
fn version() -> Result<String> {
  Ok(env!("CARGO_PKG_VERSION").to_string())
}
