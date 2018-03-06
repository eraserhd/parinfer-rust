use std::collections::HashMap;
use std::borrow::Cow;

pub type LineNumber = usize;
pub type Column = usize;
type Delta = i64;

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
// State Structure (was Result)
//------------------------------------------------------------------------------

// This represents the running result. As we scan through each character
// of a given text, we mutate this structure to update the state of our
// system.

struct Paren<'a> {
    line_no: LineNumber,
    ch: &'a str,
    x: Column,
    indent_delta: i32,
    arg_x: Option<Column>
}

struct ParenTrailClamped<'a> {
    start_x: Column,
    end_x: Column,
    openers: Vec<Paren<'a>>
}

struct ParenTrail<'a> {
    line_no: Option<LineNumber>,
    start_x: Option<Column>,
    end_x: Option<Column>,
    openers: Vec<Paren<'a>>,
    clamped: Option<ParenTrailClamped<'a>>
}

pub enum Mode {
    Indent,
    Paren
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum TrackingArgTabStop {
    NotSearching,
    Space,
    Arg
}

pub struct State<'a> {
    mode: Mode,
    smart: bool,

    orig_text: &'a str,
    orig_cursor_x: Option<Column>,
    orig_cursor_line: Option<LineNumber>,

    input_lines: Vec<&'a str>,
    input_line_no: LineNumber,
    input_x: Column,

    lines: Vec<Cow<'a, str>>,
    line_no: LineNumber,
    ch: &'a str,
    x: Column,
    indent_x: Option<Column>,

    paren_stack: Vec<Paren<'a>>,

    paren_trail: ParenTrail<'a>,

    return_parens: bool,

    cursor_x: Option<Column>,
    cursor_line: Option<LineNumber>,

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

    indent_delta: i64,

    tracking_arg_tab_stop: TrackingArgTabStop,

    error_pos_cache: HashMap<ErrorType, Error>
}

fn initial_paren_trail<'a>() -> ParenTrail<'a> {
    ParenTrail {
        line_no: None,
        start_x: None,
        end_x: None,
        openers: vec![], 
        clamped: None
    }
}

fn get_initial_result<'a>(text: &'a str, options: Options<'a>, mode: Mode, smart: bool) -> State<'a> {
    State {
        mode: mode,
        smart: smart,

        orig_text: text,
        orig_cursor_x: options.cursor_x,
        orig_cursor_line: options.cursor_line,

        input_lines: text.lines().collect(), 
        input_line_no: 0,
        input_x: 0,

        lines: vec![],
        line_no: 0,
        ch: &text[0..0],
        x: 0,
        indent_x: None,

        paren_stack: vec![],

        paren_trail: initial_paren_trail(),

        return_parens: false,

        cursor_x: options.cursor_x,
        cursor_line: options.cursor_line,

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

        indent_delta: 0,

        tracking_arg_tab_stop: TrackingArgTabStop::NotSearching,

        error_pos_cache: HashMap::new()
    }
}

//------------------------------------------------------------------------------
// Possible Errors
//------------------------------------------------------------------------------

#[derive(PartialEq, Eq, Hash)]
pub enum ErrorType {
    QuoteDanger,
    EolBackslash,
    UnclosedQuote,
    UnclosedParen,
    UnmatchedCloseParen,
    UnmatchedOpenParen,
    LeadingCloseParen,
    Unhandled,

    Restart
}

pub struct Error {
    name: ErrorType,
    message: &'static str,
    line_no: LineNumber,
    x: Column
}

pub type Result<T> = std::result::Result<T, Error>;

fn error_message(error: ErrorType) -> &'static str {
    match error {
        ErrorType::QuoteDanger => "Quotes must balanced inside comment blocks.",
        ErrorType::EolBackslash => "Line cannot end in a hanging backslash.",
        ErrorType::UnclosedQuote => "String is missing a closing quote.",
        ErrorType::UnclosedParen => "Unclosed open-paren.",
        ErrorType::UnmatchedCloseParen => "Unmatched close-paren.",
        ErrorType::UnmatchedOpenParen => "Unmatched open-paren.",
        ErrorType::LeadingCloseParen => "Line cannot lead with a close-paren.",
        ErrorType::Unhandled => "Unhandled error.",
        
        ErrorType::Restart => "Restart requested (you shouldn't see this)."
    }
}

fn cache_error_pos(result: &mut State, error: ErrorType) {
    unimplemented!();
}

fn error(result: &mut State, name: ErrorType) -> Result<()> {
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

fn is_cursor_affected<'a>(result: &State<'a>, start: Column, end: Column) -> bool {
    match result.cursor_x {
        Some(x) if x == start && x == end => x == 0,
        Some(x) => x >= end,
        None => false
    }
}

fn shift_cursor_on_edit<'a>(result: &mut State<'a>, line_no: LineNumber, start: Column, end: Column, replace: &str) {
    let old_length = end - start;
    let new_length = replace.len();
    let dx = new_length as Delta - old_length as Delta;

    if let (Some(cursor_x), Some(cursor_line)) = (result.cursor_x, result.cursor_line) {
        if dx != 0 && cursor_line == line_no && is_cursor_affected(result, start, end) {
            result.cursor_x = Some(((cursor_x as Delta) + dx) as usize);
        }
    }
}

fn replace_within_line<'a>(result: &mut State<'a>, line_no: LineNumber, start: Column, end: Column, replace: &str) {
    let line = result.lines[line_no].clone();
    let new_line = replace_within_string(&line, start, end, replace);
    result.lines[line_no] = Cow::from(new_line);

    shift_cursor_on_edit(result, line_no, start, end, replace);
}

fn insert_within_line<'a>(result: &mut State<'a>, line_no: LineNumber, idx: Column, insert: &str) {
    replace_within_line(result, line_no, idx, idx, insert);
}

fn init_line<'a>(result: &mut State<'a>) {
    result.x = 0;
    result.line_no += 1;

    // reset line-specific state
    result.indent_x = None;
    result.comment_x = None;
    result.indent_delta = 0;

    result.error_pos_cache.remove(&ErrorType::UnmatchedCloseParen);
    result.error_pos_cache.remove(&ErrorType::UnmatchedOpenParen);
    result.error_pos_cache.remove(&ErrorType::LeadingCloseParen);

    result.tracking_arg_tab_stop = TrackingArgTabStop::NotSearching;
    result.tracking_indent = !result.is_in_str;
}

fn commit_char<'a>(result: &mut State<'a>, orig_ch: &'a str) {
    let ch = result.ch;
    if orig_ch != ch {
        let line_no = result.line_no;
        let x = result.x;
        replace_within_line(result, line_no, x, x + orig_ch.len(), ch);
        result.indent_delta -= orig_ch.len() as Delta - ch.len() as Delta;
    }
    result.x += ch.len();
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

fn is_valid_close_paren<'a>(paren_stack: &Vec<Paren<'a>>, ch: &'a str) -> bool {
    if paren_stack.is_empty() {
        return false;
    }
    if let Some(paren) = peek(paren_stack, 0) {
        if let Some(close) = match_paren(ch) {
            if paren.ch == close {
                return true;
            }
        }
    }
    false
}

fn is_whitespace<'a>(result: &State<'a>) -> bool {
    !result.is_escaped && (result.ch == BLANK_SPACE || result.ch == DOUBLE_SPACE)
}

fn is_closable<'a>(result: &State<'a>) -> bool {
    let ch = result.ch;
    let closer = is_close_paren(ch) && !result.is_escaped;
    return result.is_in_code && !is_whitespace(result) && ch != "" && !closer;
}

//------------------------------------------------------------------------------
// Advanced operations on characters
//------------------------------------------------------------------------------

fn check_cursor_holding<'a>(result: &State<'a>) -> Result<bool> {
    unimplemented!();
}

fn track_arg_tab_stop<'a>(result: &mut State<'a>, state: TrackingArgTabStop) {
    if state == TrackingArgTabStop::Space {
        if result.is_in_code && is_whitespace(result) {
            result.tracking_arg_tab_stop = TrackingArgTabStop::Arg;
        }
    } else if state == TrackingArgTabStop::Arg {
        if !is_whitespace(result) {
            let opener = result.paren_stack.last_mut().unwrap();
            opener.arg_x = Some(result.x);
            result.tracking_arg_tab_stop = TrackingArgTabStop::NotSearching;
        }
    }
}

//------------------------------------------------------------------------------
// Literal character events
//------------------------------------------------------------------------------

fn on_open_paren<'a>(result: &mut State<'a>) {
    if result.is_in_code {
    }

    if result.return_parens {
    }
    unimplemented!();
}

// set_closer

fn on_matched_close_paren<'a>(result: &mut State<'a>) {
    unimplemented!();
}

fn on_unmatched_close_paren<'a>(result: &mut State<'a>) -> Result<()> {
    unimplemented!();
}

fn on_close_paren<'a>(result: &mut State<'a>) -> Result<()> {
    if result.is_in_code {
        if is_valid_close_paren(&result.paren_stack, result.ch) {
            on_matched_close_paren(result);
        } else {
            on_unmatched_close_paren(result)?;
        }
    }

    Ok(())
}

fn on_tab<'a>(result: &mut State<'a>) {
    if result.is_in_code {
        result.ch = DOUBLE_SPACE;
    }
}

fn on_semicolon<'a>(result: &mut State<'a>) {
    if result.is_in_code {
        result.is_in_comment = true;
        result.comment_x = Some(result.x);
        result.tracking_arg_tab_stop = TrackingArgTabStop::NotSearching;
    }
}

fn on_newline<'a>(result: &mut State<'a>) {
    result.is_in_comment = false;
    result.ch = "";
}

fn on_quote<'a>(result: &mut State<'a>) {
    if result.is_in_str {
        result.is_in_str = false;
    } else if result.is_in_comment {
        result.quote_danger = !result.quote_danger;
        if result.quote_danger {
            cache_error_pos(result, ErrorType::QuoteDanger);
        }
    } else {
        result.is_in_str = true;
        cache_error_pos(result, ErrorType::UnclosedQuote);
    }
}

fn on_backslash<'a>(result: &mut State<'a>) {
    result.is_escaping = true;
}

fn after_backslash<'a>(result: &mut State<'a>) -> Result<()> {
    result.is_escaping = false;
    result.is_escaped = true;

    if result.ch == NEWLINE {
        if result.is_in_code {
            return error(result, ErrorType::EolBackslash);
        }
    }

    Ok(())
}

//------------------------------------------------------------------------------
// Character dispatch
//------------------------------------------------------------------------------

fn on_char<'a>(result: &mut State<'a>) -> Result<()> {
    let mut ch = result.ch;
    result.is_escaped = false;

    if result.is_escaping      { after_backslash(result)?; }
    else if is_open_paren(ch)  { on_open_paren(result); }
    else if is_close_paren(ch) { on_close_paren(result)?; }
    else if ch == DOUBLE_QUOTE { on_quote(result); }
    else if ch == SEMICOLON    { on_semicolon(result); }
    else if ch == BACKSLASH    { on_backslash(result); }
    else if ch == TAB          { on_tab(result); }
    else if ch == NEWLINE      { on_newline(result); }

    ch = result.ch;

    result.is_in_code = !result.is_in_comment && !result.is_in_str;

    if is_closable(result) {
        let line_no = result.line_no;
        let x = result.x;
        reset_paren_trail(result, line_no, x + ch.len());
    }

    let state = result.tracking_arg_tab_stop;
    if state != TrackingArgTabStop::NotSearching {
        track_arg_tab_stop(result, state);
    }

    Ok(())
}

//------------------------------------------------------------------------------
// Cursor functions
//------------------------------------------------------------------------------

fn is_cursor_left_of<'a>(cursor_x: Option<Column>, cursor_line: Option<LineNumber>,
                         x: Option<Column>, line_no: Option<LineNumber>) -> bool {
  if let (Some(x), Some(cursor_x)) = (x, cursor_x) {
    cursor_line == line_no && cursor_x <= x // inclusive since (cursorX = x) implies (x-1 < cursor < x)
  } else {
    false
  }
}

fn is_cursor_on_right<'a>(result: &State<'a>) -> bool {
    unimplemented!();
}

fn is_cursor_in_comment<'a>(result: &State<'a>) -> bool {
    unimplemented!();
}

fn handle_change_delta<'a>(result: &State<'a>) {
    unimplemented!();
}

//------------------------------------------------------------------------------
// Paren Trail functions
//------------------------------------------------------------------------------

fn reset_paren_trail<'a>(result: &mut State<'a>, line_no: LineNumber, x: Column) {
    unimplemented!();
}

fn clamp_paren_trail_to_cursor<'a>(result: &mut State<'a>) {
    unimplemented!();
}

fn pop_paren_trail<'a>(result: &mut State<'a>) {
    unimplemented!();
}

fn get_parent_opener_index<'a>(result: &mut State<'a>, index_x: u32) -> u32 {
    unimplemented!();
}

fn correct_paren_trail<'a>(result: &mut State<'a>, index_x: u32) {
    unimplemented!();
}

fn clean_paren_trail<'a>(result: &mut State<'a>) {
    unimplemented!();
}

fn append_paren_trail<'a>(result: &mut State<'a>) {
    unimplemented!();
}

fn invalidate_paren_trail<'a>(result: &mut State<'a>) {
    unimplemented!();
}

fn check_unmatched_outside_paren_trail<'a>(result: &mut State<'a>) -> Result<()> {
    unimplemented!();
}

fn finish_new_paren_trail<'a>(result: &mut State<'a>) {
    unimplemented!();
}

//------------------------------------------------------------------------------
// Indentation functions
//------------------------------------------------------------------------------

fn change_indent<'a>(result: &mut State<'a>, delta: i32) {
    unimplemented!();
}

fn correct_indent<'a>(result: &mut State<'a>) {
    unimplemented!();
}

fn on_indent<'a>(result: &mut State<'a>) -> Result<()> {
    unimplemented!();
}

fn check_leading_close_paren<'a>(result: &mut State<'a>) -> Result<()> {
    unimplemented!();
}

fn on_leading_close_paren<'a>(result: &mut State<'a>) -> Result<()> {
    unimplemented!();
}

fn shift_comment_line<'a>(result: &mut State<'a>) {
    unimplemented!();
}

fn on_comment_line<'a>(result: &mut State<'a>) {
    unimplemented!();
}

fn check_indent<'a>(result: &mut State<'a>) -> Result<()> {
    if is_close_paren(result.ch) {
        on_leading_close_paren(result)?;
    } else if result.ch == SEMICOLON {
        // comments don't count as indentation points
        on_comment_line(result);
        result.tracking_indent = false;
    } else if result.ch != NEWLINE &&
              result.ch != BLANK_SPACE &&
              result.ch != TAB {
        on_indent(result)?;
    }

    Ok(())
}

fn get_tab_stop_line<'a>(result: &State<'a>) -> LineNumber {
    match result.selection_start_line {
        Some(line) => line,
        None => result.cursor_line.unwrap()
    }
}

fn set_tab_stops<'a>(result: &mut State<'a>) {
    if get_tab_stop_line(result) != result.line_no {
        return;
    }

    unimplemented!();
}

//------------------------------------------------------------------------------
// High-level processing functions
//------------------------------------------------------------------------------

fn process_char<'a>(result: &mut State<'a>, ch: &'a str) -> Result<()> {
    let orig_ch = ch;

    result.ch = ch;
    result.skip_char = false;

    handle_change_delta(result);

    if result.tracking_indent {
        check_indent(result)?;
    }

    if result.skip_char {
        result.ch = "";
    } else {
        on_char(result)?;
    }

    commit_char(result, orig_ch);

    Ok(())
}

fn process_line<'a>(result: &mut State<'a>, line_no: usize) -> Result<()> {
    init_line(result);
    result.lines.push(Cow::from(result.input_lines[line_no]));

    set_tab_stops(result);

    for x in 0..result.input_lines[line_no].len() {
        result.input_x = x;
        let ch = &result.input_lines[line_no][x..x];
        process_char(result, ch)?;
    }
    process_char(result, NEWLINE)?;

    if !result.force_balance {
        check_unmatched_outside_paren_trail(result)?;
        check_leading_close_paren(result)?;
    }

    if Some(result.line_no) == result.paren_trail.line_no {
        finish_new_paren_trail(result);
    }

    Ok(())
}

fn finalize_result<'a>(result: &mut State<'a>) -> Result<()> {
    unimplemented!();
}

fn process_error<'a>(result: &mut State<'a>) -> Result<()> {
    unimplemented!();
}

fn process_text<'a>(text: &'a str, options: Options<'a>, mode: Mode, smart: bool) -> Result<State<'a>> {
    let mut result = get_initial_result(text, options, mode, smart);

    for i in 0..result.input_lines.len() {
        result.input_line_no = i;
        process_line(&mut result, i)?;
    }
    finalize_result(&mut result)?;

    Ok(result)
}

//------------------------------------------------------------------------------
// Public API
//------------------------------------------------------------------------------

fn public_result<'a>(result: State<'a>) -> State<'a> {
    unimplemented!();
}

pub fn indent_mode<'a>(text: &'a str, options: Options<'a>) -> Result<State<'a>> {
    process_text(text, options, Mode::Indent, false).map(public_result)
}

pub fn paren_mode<'a>(text: &'a str, options: Options<'a>) -> Result<State<'a>> {
    process_text(text, options, Mode::Paren, false).map(public_result)
}

pub fn smart_mode<'a>(text: &'a str, options: Options<'a>) -> Result<State<'a>> {
    let smart = options.selection_start_line == None;
    process_text(text, options, Mode::Indent, smart).map(public_result)
}
