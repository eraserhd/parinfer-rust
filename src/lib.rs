extern crate regex;

use regex::Regex;
use std::collections::HashMap;

pub enum Mode {
    INDENT,
    PAREN
}

const BACKSLASH : &'static str = "\\";
const BLANK_SPACE : &'static str = " ";
const DOUBLE_SPACE : &'static str = "  ";
const DOUBLE_QUOTE : &'static str = "\"";
const NEWLINE : &'static str = "\n";
const SEMICOLON : &'static str = ";";
const TAB : &'static str = "\t";

const LINE_ENDING_REGEX : &'static str = r"\r?\n";

fn match_paren(paren: &str) -> Option<&'static str> {
    match paren {
        "{" => Some("}"),
        "}" => Some("{"),
        "[" => Some("]"),
        "]" => Some("["),
        "(" => Some(")"),
        ")" => Some("("),
        _ => None
    }
}

#[cfg(test)]
#[test]
fn match_paren_works() {
    assert_eq!(match_paren("}"), Some("{"));
    assert_eq!(match_paren("x"), None);
}

fn is_open_paren(paren: &str) -> bool {
    match paren {
        "{" | "[" | "(" => true,
        _ => false
    }
}

#[cfg(test)]
#[test]
fn is_open_paren_works() {
    assert!(is_open_paren("("));
    assert!(!is_open_paren("}"));
}

fn is_close_paren(paren: &str) -> bool {
    match paren {
        "}" | "]" | ")" => true,
        _ => false
    }
}

//------------------------------------------------------------------------------
// Options Structure
//------------------------------------------------------------------------------

pub struct Change<'a> {
    x: u32,
    line_no: u32,
    old_text: &'a str,
    new_text: &'a str,
}

struct TransformedChange<'a> {
    x: u32,
    line_no: u32,
    old_text: &'a str,
    new_text: &'a str,
    old_end_x: u32,
    new_end_x: u32,
    new_end_line_no: u32,
    lookup_line_no: u32,
    lookup_x: u32
}

fn transform_change<'a>(change: &Change<'a>) -> TransformedChange<'a> {
    unimplemented!();
}

fn transform_changes<'a>(changes: &Vec<Change<'a>>) -> HashMap<(u32, u32), TransformedChange<'a>> {
    unimplemented!();
}

pub struct Options<'a> {
    cursor_x: u32,
    cursor_line: u32,
    changes: Vec<Change<'a>>,
    partial_result: bool,
    force_balance: bool
}

pub struct Result<'a> {
    mode: Mode,
    orig_text: &'a str,
    changes: HashMap<(u32, u32), TransformedChange<'a>>
}

fn get_initial_result<'a>(text: &'a str, options: Options<'a>, mode: Mode) -> Result<'a> {
    unimplemented!();
}

//------------------------------------------------------------------------------
// Possible Errors
//------------------------------------------------------------------------------

pub enum Error {
    ERROR_QUOTE_DANGER,
    ERROR_EOL_BACKSLASH,
    ERROR_UNCLOSED_QUOTE,
    ERROR_UNCLOSED_PAREN,
    ERROR_UNMATCHED_CLOSE_PAREN,
    ERROR_UNMATCHED_OPEN_PAREN,
    ERROR_UNHANDLED 
}

fn error_message(error: Error) -> &'static str {
    unimplemented!();
}

fn cache_error_pos(result: &mut Result, error: Error) {
    unimplemented!();
}

fn error(result: &mut Result, name: Error) {
    unimplemented!();
}

//------------------------------------------------------------------------------
// String Operations
//------------------------------------------------------------------------------

//------------------------------------------------------------------------------
// Line operations
//------------------------------------------------------------------------------

//------------------------------------------------------------------------------
// Misc Utils
//------------------------------------------------------------------------------

//------------------------------------------------------------------------------
// Cursor functions
//------------------------------------------------------------------------------

//------------------------------------------------------------------------------
// Character functions
//------------------------------------------------------------------------------

//------------------------------------------------------------------------------
// Paren Trail functions
//------------------------------------------------------------------------------

//------------------------------------------------------------------------------
// Indentation functions
//------------------------------------------------------------------------------

//------------------------------------------------------------------------------
// High-level processing functions
//------------------------------------------------------------------------------

//------------------------------------------------------------------------------
// Public API
//------------------------------------------------------------------------------
