use std::collections::HashMap;

pub type LineNumber = usize;
pub type Column = usize;

//------------------------------------------------------------------------------
// Constants / Predicates
//------------------------------------------------------------------------------

const BACKSLASH : &'static str = "\\";
const BLANK_SPACE : &'static str = " ";
const DOUBLE_SPACE : &'static str = "  ";
const DOUBLE_QUOTE : &'static str = "\"";
const NEWLINE : &'static str = "\n";
const SEMICOLON : &'static str = ";";
const TAB : &'static str = "\t";

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

//------------------------------------------------------------------------------
// Options Structure
//------------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub struct Change<'a> {
    x: Column,
    line_no: LineNumber,
    old_text: &'a str,
    new_text: &'a str,
}

struct TransformedChange<'a> {
    x: Column,
    line_no: LineNumber,
    old_text: &'a str,
    new_text: &'a str,
    old_end_x: Column,
    new_end_x: Column,
    new_end_line_no: LineNumber,
    lookup_line_no: LineNumber,
    lookup_x: Column
}

fn transform_change<'a>(change: &Change<'a>) -> TransformedChange<'a> {
    unimplemented!();
}

fn transform_changes<'a>(changes: &Vec<Change<'a>>) -> HashMap<(LineNumber, Column), TransformedChange<'a>> {
    unimplemented!();
}

pub struct Options<'a> {
    cursor_x: Option<Column>,
    cursor_line: Option<LineNumber>,
    prev_cursor_x: Option<Column>,
    prev_cursor_line: Option<LineNumber>,
    selection_start_line: Option<LineNumber>,
    changes: Vec<Change<'a>>,
    partial_result: bool,
    force_balance: bool,
    return_parens: bool
}

//------------------------------------------------------------------------------
// Result Structure
//------------------------------------------------------------------------------

// This represents the running result. As we scan through each character
// of a given text, we mutate this structure to update the state of our
// system.

struct Paren {
    line_no: LineNumber,
    ch: char,
    x: Column,
    indent_delta: i32
}

struct ParenTrailClamped {
    start_x: Column,
    end_x: Column,
    openers: Vec<Paren>
}

struct ParenTrail {
    line_no: Option<LineNumber>,
    start_x: Option<Column>,
    end_x: Option<Column>,
    openers: Vec<Paren>,
    clamped: Option<ParenTrailClamped>
}

pub enum Mode {
    Indent,
    Paren
}

enum TrackingArgTabStop {
    NotSearching,
    Space,
    Arg
}

pub struct Result<'a> {
    mode: Mode,
    smart: bool,

    orig_text: &'a str,
    orig_cursor_x: Option<Column>,
    orig_cursor_line: Option<LineNumber>,

    input_lines: Vec<&'a str>,
    input_line_no: LineNumber,
    input_x: Column,

    lines: Vec<String>,
    line_no: Option<LineNumber>,
    ch: &'a str,
    x: Column,
    indent_x: Option<Column>,

    paren_stack: Vec<Paren>,

    paren_trail: ParenTrail,

    return_parens: bool,

    selection_start_line: Option<LineNumber>,

    changes: HashMap<(LineNumber, Column), TransformedChange<'a>>,

    is_in_code: bool,
    is_escaping: bool,
    is_escaped: bool,
    is_in_str: bool,
    is_in_comment: bool,
    comment_x: Option<Column>,

    quote_danger: bool,
    tracking_indent: bool,
    skip_char: bool,
    success: bool,
    partial_result: bool,
    force_balance: bool,

    tracking_arg_tab_stop: TrackingArgTabStop
}

fn initial_paren_trail() -> ParenTrail {
    ParenTrail {
        line_no: None,
        start_x: None,
        end_x: None,
        openers: vec![], 
        clamped: None
    }
}

fn get_initial_result<'a>(text: &'a str, options: Options<'a>, mode: Mode, smart: bool) -> Result<'a> {
    Result {
        mode: mode,
        smart: smart,

        orig_text: text,
        orig_cursor_x: options.cursor_x,
        orig_cursor_line: options.cursor_line,

        input_lines: text.lines().collect(), 
        input_line_no: 0,
        input_x: 0,

        lines: vec![],
        line_no: None,
        ch: &text[0..0],
        x: 0,
        indent_x: None,

        paren_stack: vec![],

        paren_trail: initial_paren_trail(),

        return_parens: false,

        selection_start_line: None,

        changes: transform_changes(&options.changes),

        is_in_code: true,
        is_escaping: false,
        is_escaped: false,
        is_in_str: false,
        is_in_comment: false,
        comment_x: None,

        quote_danger: false,
        tracking_indent: false,
        skip_char: false,
        success: false,
        partial_result: false,
        force_balance: false,

        tracking_arg_tab_stop: TrackingArgTabStop::NotSearching
    }
}

//------------------------------------------------------------------------------
// Possible Errors
//------------------------------------------------------------------------------

pub enum Error {
    QuoteDanger,
    EolBackslash,
    UnclosedQuote,
    UnclosedParen,
    UnmatchedCloseParen,
    UnmatchedOpenParen,
    LeadingCloseParen,
    Unhandled 
}

fn error_message(error: Error) -> &'static str {
    match error {
        Error::QuoteDanger => "Quotes must balanced inside comment blocks.",
        Error::EolBackslash => "Line cannot end in a hanging backslash.",
        Error::UnclosedQuote => "String is missing a closing quote.",
        Error::UnclosedParen => "Unclosed open-paren.",
        Error::UnmatchedCloseParen => "Unmatched close-paren.",
        Error::UnmatchedOpenParen => "Unmatched open-paren.",
        Error::LeadingCloseParen => "Line cannot lead with a close-paren.",
        Error::Unhandled => "Unhandled error.",
    }
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
    String::from(&orig[0..start]) + replace + &orig[end..]
}

#[cfg(test)]
#[test]
fn replace_within_string_works() {
    assert_eq!(replace_within_string("aaa", 0, 2, ""), "a");
    assert_eq!(replace_within_string("aaa", 0, 1, "b"), "baa");
    assert_eq!(replace_within_string("aaa", 0, 2, "b"), "ba");
}

fn repeat_string(text: &str, n: usize) -> String {
    String::from(text).repeat(n)
}

#[cfg(test)]
#[test]
fn repeat_string_works() {
    assert_eq!(repeat_string("a", 2), "aa");
    assert_eq!(repeat_string("aa", 3), "aaaaaa");
    assert_eq!(repeat_string("aa", 0), "");
    assert_eq!(repeat_string("", 0), "");
    assert_eq!(repeat_string("", 5), "");
}

fn get_line_ending(text: &str) -> &'static str {
    if text.chars().any(|ch| ch == '\r') {
        "\r\n"
    } else {
        "\n"
    }
}

#[cfg(test)]
#[test]
fn get_line_ending_works() {
    assert_eq!(get_line_ending("foo\nbar"), "\n");
    assert_eq!(get_line_ending("foo\r\nbar"), "\r\n");
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

fn commit_char<'a>(result: &mut Result<'a>, orig_ch: &'a str) {
    unimplemented!();
}

//------------------------------------------------------------------------------
// Misc Utils
//------------------------------------------------------------------------------

fn clamp<T : Clone + Ord>(val: T, min_n: Option<T>, max_n: Option<T>) -> T {
    if let Some(low) = min_n {
        if low >= val {
            return low;
        }
    }
    if let Some(high) = max_n {
        if high <= val {
            return high;
        }
    }
    val
}

#[cfg(test)]
#[test]
fn clamp_works() {
    assert_eq!(clamp(1, Some(3), Some(5)), 3);
    assert_eq!(clamp(9, Some(3), Some(5)), 5);
    assert_eq!(clamp(1, Some(3), None), 3);
    assert_eq!(clamp(5, Some(3), None), 5);
    assert_eq!(clamp(1, None, Some(5)), 1);
    assert_eq!(clamp(9, None, Some(5)), 5);
    assert_eq!(clamp(1, None, None), 1);
}

fn peek<T>(array: &Vec<T>, i: usize) -> Option<&T> {
    if i >= array.len() {
        None
    } else {
        Some(&array[array.len() - 1 - i])
    }
}

#[cfg(test)]
#[test]
fn peek_works() {
    assert_eq!(peek(&vec!['a'], 0), Some(&'a'));
    assert_eq!(peek(&vec!['a'], 1), None);
    assert_eq!(peek(&vec!['a', 'b', 'c'], 0), Some(&'c'));
    assert_eq!(peek(&vec!['a', 'b', 'c'], 1), Some(&'b'));
    assert_eq!(peek(&vec!['a', 'b', 'c'], 5), None);
    let empty : Vec<char> = vec![];
    assert_eq!(peek(&empty, 0), None);
    assert_eq!(peek(&empty, 1), None);
}

//------------------------------------------------------------------------------
// Questions about characters
//------------------------------------------------------------------------------

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

fn is_valid_close_paren<'a>(paren_stack: &Vec<Paren>, ch: char) {
    unimplemented!();
}

fn is_whitespace<'a>(result: &Result<'a>) -> bool {
    unimplemented!();
}

fn is_closable<'a>(result: &Result<'a>) -> bool {
    unimplemented!();
}

//------------------------------------------------------------------------------
// Advanced operations on characters
//------------------------------------------------------------------------------

fn check_cursor_holding<'a>(result: &Result<'a>) -> bool {
    unimplemented!();
}

fn track_arg_tab_stop<'a>(result: &Result<'a>, state: TrackingArgTabStop) -> bool {
    unimplemented!();
}

//------------------------------------------------------------------------------
// Literal character events
//------------------------------------------------------------------------------

fn on_open_paren<'a>(result: &mut Result<'a>) {
    unimplemented!();
}

// set_closer

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

//------------------------------------------------------------------------------
// Character dispatch
//------------------------------------------------------------------------------

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

fn process_char<'a>(result: &mut Result<'a>, ch: &'a str) {
    let orig_ch = ch;

    result.ch = ch;
    result.skip_char = false;

    handle_change_delta(result);

    if result.tracking_indent {
        check_indent(result);
    }

    if result.skip_char {
        result.ch = "";
    } else {
        on_char(result);
    }

    commit_char(result, orig_ch);
}

fn process_line<'a>(result: &mut Result<'a>, line_no: usize) {
    unimplemented!();
}

fn finalize_result<'a>(result: &mut Result<'a>) {
    unimplemented!();
}

// process_error

fn process_text<'a>(text: &'a str, options: Options<'a>, mode: Mode, smart: bool) -> Result<'a> {
    let mut result = get_initial_result(text, options, mode, smart);

    for i in 0..result.input_lines.len() {
        result.input_line_no = i;
        process_line(&mut result, i);
    }
    finalize_result(&mut result);

    result
}

//------------------------------------------------------------------------------
// Public API
//------------------------------------------------------------------------------

fn public_result<'a>(result: Result<'a>) -> Result<'a> {
    unimplemented!();
}

pub fn indent_mode<'a>(text: &'a str, options: Options<'a>) {
    public_result(process_text(text, options, Mode::Indent, false));
}

pub fn paren_mode<'a>(text: &'a str, options: Options<'a>) {
    public_result(process_text(text, options, Mode::Paren, false));
}

pub fn smart_mode<'a>(text: &'a str, options: Options<'a>) {
    let smart = options.selection_start_line == None;
    public_result(process_text(text, options, Mode::Indent, smart));
}
