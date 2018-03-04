extern crate regex;

use regex::Regex;
use std::collections::HashMap;

pub enum Mode {
    Indent,
    Paren
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

//------------------------------------------------------------------------------
// Result Structure
//------------------------------------------------------------------------------

// This represents the running result. As we scan through each character
// of a given text, we mutate this structure to update the state of our
// system.

pub struct Paren {
    line_no: u32,
    ch: char,
    x: u32,
    paren_stack: Vec<Paren>,
    indent_delta: i32
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
    ErrorQuoteDanger,
    ErrorEolBackslash,
    ErrorUnclosedQuote,
    ErrorUnclosedParen,
    ErrorUnmatchedCloseParen,
    ErrorUnmatchedOpenParen,
    ErrorUnhandled 
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

fn replace_within_string(orig: &str, start: usize, end: usize, replace: &str) -> String {
    unimplemented!();
}

fn repeat_string(text: &str, n: usize) -> String {
    unimplemented!();
}

fn get_line_ending(text: &str) -> &'static str {
    unimplemented!();
}

//------------------------------------------------------------------------------
// Line operations
//------------------------------------------------------------------------------

fn is_cursor_affected<'a>(result: &Result<'a>, start: u32, end: u32) -> bool {
    unimplemented!();
}

fn shift_cursor_on_edit<'a>(result: &mut Result<'a>, line_no: u32, start: u32, end: u32, replace: &str) {
    unimplemented!();
}

fn replace_within_line<'a>(result: &mut Result<'a>, line_no: u32, start: u32, end: u32, replace: &str) {
    unimplemented!();
}

fn insert_within_line<'a>(result: &mut Result<'a>, line_no: u32, idx: u32, insert: &str) {
    unimplemented!();
}

fn init_line<'a>(result: &mut Result<'a>, line: &str) {
    unimplemented!();
}

fn commit_char<'a>(result: &mut Result<'a>, orig_ch: char) {
    unimplemented!();
}

//------------------------------------------------------------------------------
// Misc Utils
//------------------------------------------------------------------------------

fn clamp<T : Clone + Ord>(val: T, min_n: Option<T>, max_n: Option<T>) -> T {
    unimplemented!();
}

fn peek<T>(array: &Vec<T>, i: usize) -> &T {
    unimplemented!();
}

//------------------------------------------------------------------------------
// Character functions
//------------------------------------------------------------------------------

fn is_valid_close_paren<'a>(paren_stack: &Vec<Paren>, ch: char) {
    unimplemented!();
}

fn on_open_paren<'a>(result: &mut Result<'a>) {
    unimplemented!();
}

fn on_matched_close_paren<'a>(result: &mut Result<'a>) {
    unimplemented!();
}

fn on_unmatched_close_paren<'a>(result: &mut Result<'a>) {
    unimplemented!();
}

fn on_close_paren<'a>(result: &mut Result<'a>) {
    unimplemented!();
}

fn on_tab<'a>(result: &mut Result<'a>) {
    unimplemented!();
}

fn on_semicolon<'a>(result: &mut Result<'a>) {
    unimplemented!();
}

fn on_newline<'a>(result: &mut Result<'a>) {
    unimplemented!();
}

fn on_quote<'a>(result: &mut Result<'a>) {
    unimplemented!();
}

fn on_backslash<'a>(result: &mut Result<'a>) {
    unimplemented!();
}

fn after_backslash<'a>(result: &mut Result<'a>) {
    unimplemented!();
}

fn on_char<'a>(result: &mut Result<'a>) {
    unimplemented!();
}

//------------------------------------------------------------------------------
// Cursor functions
//------------------------------------------------------------------------------

fn is_cursor_on_left<'a>(result: &Result<'a>) -> bool {
    unimplemented!();
}

fn is_cursor_on_right<'a>(result: &Result<'a>) -> bool {
    unimplemented!();
}

fn is_cursor_in_comment<'a>(result: &Result<'a>) -> bool {
    unimplemented!();
}

fn handle_change_delta<'a>(result: &Result<'a>) {
    unimplemented!();
}

//------------------------------------------------------------------------------
// Paren Trail functions
//------------------------------------------------------------------------------

fn reset_paren_trail<'a>(result: &mut Result<'a>, line_no: u32, x: u32) {
    unimplemented!();
}

fn clamp_paren_trail_to_cursor<'a>(result: &mut Result<'a>) {
    unimplemented!();
}

fn pop_paren_trail<'a>(result: &mut Result<'a>) {
    unimplemented!();
}

fn get_parent_opener_index<'a>(result: &mut Result<'a>, index_x: u32) -> u32 {
    unimplemented!();
}

fn correct_paren_trail<'a>(result: &mut Result<'a>, index_x: u32) {
    unimplemented!();
}

fn clean_paren_trail<'a>(result: &mut Result<'a>) {
    unimplemented!();
}

fn append_paren_trail<'a>(result: &mut Result<'a>) {
    unimplemented!();
}

fn invalidate_paren_trail<'a>(result: &mut Result<'a>) {
    unimplemented!();
}

fn check_unmatched_outside_paren_trail<'a>(result: &mut Result<'a>) {
    unimplemented!();
}

fn finish_new_paren_trail<'a>(result: &mut Result<'a>) {
    unimplemented!();
}

//------------------------------------------------------------------------------
// Indentation functions
//------------------------------------------------------------------------------

fn change_indent<'a>(result: &mut Result<'a>, delta: i32) {
    unimplemented!();
}

fn correct_indent<'a>(result: &mut Result<'a>) {
    unimplemented!();
}

fn on_indent<'a>(result: &mut Result<'a>) {
    unimplemented!();
}

fn on_leading_close_paren<'a>(result: &mut Result<'a>) {
    unimplemented!();
}

fn shift_comment_line<'a>(result: &mut Result<'a>) {
    unimplemented!();
}

fn check_indent<'a>(result: &mut Result<'a>) {
    unimplemented!();
}

fn init_indent<'a>(result: &mut Result<'a>) {
    unimplemented!();
}

fn set_tab_stops<'a>(result: &mut Result<'a>) {
    unimplemented!();
}

//------------------------------------------------------------------------------
// High-level processing functions
//------------------------------------------------------------------------------

fn process_char<'a>(result: &mut Result<'a>, ch: char) {
    unimplemented!();
}

fn process_line<'a>(reuslt: &mut Result<'a>, line_no: u32) {
    unimplemented!();
}

fn finalize_result<'a>(result: &mut Result<'a>) {
    unimplemented!();
}

// process_error

fn process_text<'a>(text: &'a str, options: Options<'a>, mode: Mode) {
    unimplemented!();
}

//------------------------------------------------------------------------------
// Public API
//------------------------------------------------------------------------------

fn public_result<'a>(result: Result<'a>) -> Result<'a> {
    unimplemented!();
}

fn indent_mode<'a>(text: &'a str, options: Options<'a>) {
    unimplemented!();
}

fn paren_mode<'a>(text: &'a str, options: Options<'a>) {
    unimplemented!();
}
