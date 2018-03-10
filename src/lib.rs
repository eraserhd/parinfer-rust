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
    let new_lines : Vec<&'a str> = change.new_text.lines().collect();
    let old_lines : Vec<&'a str> = change.old_text.lines().collect();

    // single line case:
    //     (defn foo| [])
    //              ^ newEndX, newEndLineNo
    //           +++

    // multi line case:
    //     (defn foo
    //           ++++
    //        "docstring."
    //     ++++++++++++++++
    //       |[])
    //     ++^ newEndX, newEndLineNo

    let last_old_line_len = old_lines[old_lines.len()-1].len();
    let last_new_line_len = new_lines[new_lines.len()-1].len();

    let old_end_x = (if old_lines.len() == 1 { change.x } else { 0 }) + last_old_line_len;
    let new_end_x = (if new_lines.len() == 1 { change.x } else { 0 }) + last_new_line_len;
    let new_end_line_no = change.line_no + (new_lines.len()-1);

    TransformedChange {
        x: change.x,
        line_no: change.line_no,
        old_text: change.old_text,
        new_text: change.new_text,

        old_end_x: old_end_x,
        new_end_x: new_end_x,
        new_end_line_no: new_end_line_no,

        lookup_line_no: new_end_line_no,
        lookup_x: new_end_x
    }
}

fn transform_changes<'a>(changes: &Vec<Change<'a>>) -> HashMap<(LineNumber, Column), TransformedChange<'a>> {
    let mut lines : HashMap<(LineNumber, Column), TransformedChange<'a>> = HashMap::new();
    for change in changes {
        let transformed_change = transform_change(change);
        lines.insert((transformed_change.lookup_line_no, transformed_change.lookup_x), transformed_change);
    }
    lines
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

#[derive(Clone)]
struct Paren<'a> {
    line_no: LineNumber,
    ch: &'a str,
    x: Column,
    indent_delta: Delta,
    max_child_indent: Option<Column>,
    arg_x: Option<Column>,
    input_line_no: LineNumber,
    input_x: Column
}

struct ParenTrailClamped<'a> {
    start_x: Option<Column>,
    end_x: Option<Column>,
    openers: Vec<Paren<'a>>
}

struct ParenTrail<'a> {
    line_no: Option<LineNumber>,
    start_x: Option<Column>,
    end_x: Option<Column>,
    openers: Vec<Paren<'a>>,
    clamped: Option<ParenTrailClamped<'a>>
}

#[derive(PartialEq, Eq)]
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

    max_indent: Option<Column>,
    indent_delta: i64,

    tracking_arg_tab_stop: TrackingArgTabStop,

    error: Option<Error>,
    error_pos_cache: HashMap<ErrorName, Error>
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

fn get_initial_result<'a>(text: &'a str, options: &Options<'a>, mode: Mode, smart: bool) -> State<'a> {
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

        max_indent: None,
        indent_delta: 0,

        tracking_arg_tab_stop: TrackingArgTabStop::NotSearching,

        error: None,
        error_pos_cache: HashMap::new()
    }
}

//------------------------------------------------------------------------------
// Possible Errors
//------------------------------------------------------------------------------

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum ErrorName {
    QuoteDanger,
    EolBackslash,
    UnclosedQuote,
    UnclosedParen,
    UnmatchedCloseParen,
    UnmatchedOpenParen,
    LeadingCloseParen,

    Restart
}

pub struct Error {
    name: ErrorName,
    message: &'static str,
    x: Column,
    line_no: LineNumber,
    input_x: Column,
    input_line_no: LineNumber,
}

pub type Result<T> = std::result::Result<T, Error>;

fn error_message(error: ErrorName) -> &'static str {
    match error {
        ErrorName::QuoteDanger => "Quotes must balanced inside comment blocks.",
        ErrorName::EolBackslash => "Line cannot end in a hanging backslash.",
        ErrorName::UnclosedQuote => "String is missing a closing quote.",
        ErrorName::UnclosedParen => "Unclosed open-paren.",
        ErrorName::UnmatchedCloseParen => "Unmatched close-paren.",
        ErrorName::UnmatchedOpenParen => "Unmatched open-paren.",
        ErrorName::LeadingCloseParen => "Line cannot lead with a close-paren.",
        
        ErrorName::Restart => "Restart requested (you shouldn't see this)."
    }
}

fn cache_error_pos(result: &mut State, name: ErrorName) {
    let error = Error {
        name,
        message: "",
        line_no: result.line_no,
        x: result.x,
        input_line_no: result.input_line_no,
        input_x: result.input_x
    };
    result.error_pos_cache.insert(name, error);
}

fn error(result: &mut State, name: ErrorName) -> Result<()> {
    let (line_no, x) = match (result.partial_result, result.error_pos_cache.get(&name)) {
        (true,  Some(cache)) => (cache.line_no, cache.x),
        (false, Some(cache)) => (cache.input_line_no, cache.input_x),
        (true,  None)        => (result.line_no, result.x),
        (false, None)        => (result.input_line_no, result.input_x)
    };

    let mut e = Error {
        name,
        line_no,
        x,
        message: error_message(name),
        input_line_no: result.input_line_no,
        input_x: result.input_x
    };

    if name == ErrorName::UnmatchedCloseParen {
        // extra error info for locating the open-paren that it should've matched
        if let Some(cache) = result.error_pos_cache.get(&ErrorName::UnmatchedOpenParen) {
            if let Some(opener) = peek(&result.paren_stack, 0) {
            /*
              e.extra = {
                  name: ErrorName::UnmatchedOpenParen,
                  lineNo: cache ? cache[key_line_no] : opener[key_line_no],
                  x: cache ? cache[key_x] : opener[key_x]
              };
              */
            }
        }
    } else if name == ErrorName::UnclosedParen {
        if let Some(opener) = peek(&result.paren_stack, 0) {
            e.line_no = if result.partial_result { opener.line_no } else { opener.input_line_no };
            e.x = if result.partial_result { opener.x } else { opener.input_x };
        }
    }

    Err(e)
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

    result.error_pos_cache.remove(&ErrorName::UnmatchedCloseParen);
    result.error_pos_cache.remove(&ErrorName::UnmatchedOpenParen);
    result.error_pos_cache.remove(&ErrorName::LeadingCloseParen);

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
        let opener = Paren {
            input_line_no: result.input_line_no,
            input_x: result.input_x,

            line_no: result.line_no,
            x: result.x,
            ch: result.ch,
            indent_delta: result.indent_delta,
            max_child_indent: None,

            arg_x: None
        };

        result.paren_stack.push(opener);
        result.tracking_arg_tab_stop = TrackingArgTabStop::Space;
    }
}

fn on_matched_close_paren<'a>(result: &mut State<'a>) -> Result<()> {
    let opener = (*peek(&result.paren_stack, 0).unwrap()).clone();
    if result.return_parens {
        //setCloser(opener, result.lineNo, result.x, result.ch);
    }

    result.paren_trail.end_x = Some(result.x + 1);
    result.paren_trail.openers.push(opener);

    if result.mode == Mode::Indent && result.smart && check_cursor_holding(result)? {
        let orig_start_x = result.paren_trail.start_x;
        let orig_end_x = result.paren_trail.end_x;
        let orig_openers = result.paren_trail.openers.clone();
        let x = result.x;
        let line_no = result.line_no;
        reset_paren_trail(result, line_no, x+1);
        result.paren_trail.clamped = Some(ParenTrailClamped {
            start_x:  orig_start_x,
            end_x: orig_end_x,
            openers: orig_openers
        });
    }
    result.paren_stack.pop();
    result.tracking_arg_tab_stop = TrackingArgTabStop::NotSearching;

    Ok(())
}

fn on_unmatched_close_paren<'a>(result: &mut State<'a>) -> Result<()> {
    match result.mode {
        Mode::Paren => {
            let in_leading_paren_trail = result.paren_trail.line_no == Some(result.line_no) && result.paren_trail.start_x == result.indent_x;
            let can_remove = result.smart && in_leading_paren_trail;
            if !can_remove {
                error(result, ErrorName::UnmatchedCloseParen)?;
            }
        },
        Mode::Indent => {
            if !result.error_pos_cache.contains_key(&ErrorName::UnmatchedCloseParen)  {
                cache_error_pos(result, ErrorName::UnmatchedCloseParen);
                if peek(&result.paren_stack, 0).is_some() {
                    cache_error_pos(result, ErrorName::UnmatchedOpenParen);
                    let opener = peek(&result.paren_stack, 0).unwrap();
                    if let Some(err) = result.error_pos_cache.get_mut(&ErrorName::UnmatchedOpenParen) {
                        err.input_line_no = opener.input_line_no;
                        err.input_x = opener.input_x;
                    }
                }
            }
        }

    }
    result.ch = "";

    Ok(())
}

fn on_close_paren<'a>(result: &mut State<'a>) -> Result<()> {
    if result.is_in_code {
        if is_valid_close_paren(&result.paren_stack, result.ch) {
            on_matched_close_paren(result)?;
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
            cache_error_pos(result, ErrorName::QuoteDanger);
        }
    } else {
        result.is_in_str = true;
        cache_error_pos(result, ErrorName::UnclosedQuote);
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
            return error(result, ErrorName::EolBackslash);
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
                         x: Option<Column>, line_no: LineNumber) -> bool {
  if let (Some(x), Some(cursor_x)) = (x, cursor_x) {
    cursor_line == Some(line_no) && cursor_x <= x // inclusive since (cursorX = x) implies (x-1 < cursor < x)
  } else {
    false
  }
}

fn is_cursor_right_of<'a>(cursor_x: Option<Column>, cursor_line: Option<LineNumber>,
                          x: Option<Column>, line_no: LineNumber) -> bool {
  if let (Some(x), Some(cursor_x)) = (x, cursor_x) {
    cursor_line == Some(line_no) && cursor_x > x
  } else {
    false
  }
}

fn is_cursor_in_comment<'a>(result: &State<'a>, cursor_x: Option<Column>, cursor_line: Option<LineNumber>) -> bool {
    is_cursor_right_of(cursor_x, cursor_line, result.comment_x, result.line_no)
}

fn handle_change_delta<'a>(result: &mut State<'a>) {
    if !result.changes.is_empty() && (result.smart || result.mode == Mode::Paren) {
        if let Some(change) = result.changes.get(&(result.input_line_no, result.input_x)) {
            result.indent_delta += change.new_end_x as Delta - change.old_end_x as Delta;
        }
    }
}

//------------------------------------------------------------------------------
// Paren Trail functions
//------------------------------------------------------------------------------

fn reset_paren_trail<'a>(result: &mut State<'a>, line_no: LineNumber, x: Column) {
    result.paren_trail.line_no = Some(line_no);
    result.paren_trail.start_x = Some(x);
    result.paren_trail.end_x = Some(x);
    result.paren_trail.openers = vec![];
    //FIXME:
    //result.paren_trail.clamped.start_x = None;
    //result.paren_trail.clamped.end_x = None;
    //result.paren_trail.clamped.openers = vec![];
}

fn is_cursor_clamping_paren_trail<'a>(result: &State<'a>, cursor_x: Option<Column>, cursor_line: Option<LineNumber>) -> bool {
    is_cursor_right_of(cursor_x, cursor_line, result.paren_trail.start_x, result.line_no) &&
        !is_cursor_in_comment(result, cursor_x, cursor_line)
}

// INDENT MODE: allow the cursor to clamp the paren trail
fn clamp_paren_trail_to_cursor<'a>(result: &mut State<'a>) {
    let clamping = is_cursor_clamping_paren_trail(result, result.cursor_x, result.cursor_line);
    if clamping {
        let start_x = result.paren_trail.start_x.unwrap();
        let end_x = result.paren_trail.end_x.unwrap();

        let new_start_x = std::cmp::max(start_x, result.cursor_x.unwrap());
        let new_end_x = std::cmp::max(end_x, result.cursor_x.unwrap());

        let line = &result.lines[result.line_no];
        let mut remove_count = 0;
        for i in start_x..new_start_x {
            if is_close_paren(&line[i..i]) {
                remove_count += 1;
            }
        }

        result.paren_trail.openers = (&result.paren_trail.openers[..remove_count]).to_vec();
        result.paren_trail.start_x = Some(new_start_x);
        result.paren_trail.end_x = Some(new_end_x);

        /* FIXME:
        result.paren_trail.clamped.openers = openers.slice(0, remove_count);
        result.paren_trail.clamped.startX = start_x;
        result.paren_trail.clamped.endX = end_x;
        */
    }
}

fn pop_paren_trail<'a>(result: &mut State<'a>) {
    unimplemented!();
}

fn get_parent_opener_index<'a>(result: &mut State<'a>, index_x: usize) -> usize {
    unimplemented!();
}

fn correct_paren_trail<'a>(result: &mut State<'a>, index_x: usize) {
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

fn add_indent<'a>(result: &mut State<'a>, delta: Delta) {
    let orig_indent = result.x;
    let new_indent = (orig_indent as Delta + delta) as Column;
    let indent_str = repeat_string(BLANK_SPACE, new_indent);
    let line_no = result.line_no;
    replace_within_line(result, line_no, 0, orig_indent, &indent_str);
    result.x = new_indent;
    result.indent_x = Some(new_indent);
    result.indent_delta += delta;
}

fn should_add_opener_indent<'a>(result: &State<'a>, opener: &Paren<'a>) -> bool {
    // Don't add opener.indent_delta if the user already added it.
    // (happens when multiple lines are indented together)
    opener.indent_delta != result.indent_delta
}

fn correct_indent<'a>(result: &mut State<'a>) {
    let orig_indent = result.x as Delta;
    let mut new_indent = orig_indent as Delta;
    let mut min_indent = 0;
    let mut max_indent = result.max_indent.map(|x| x as Delta);

    if let Some(opener) = peek(&result.paren_stack, 0) {
        min_indent = opener.x + 1;
        max_indent = opener.max_child_indent.map(|x| x as Delta);
        if should_add_opener_indent(result, opener) {
            new_indent += opener.indent_delta;
        }
    }

    new_indent = clamp(new_indent, Some(min_indent as Delta), max_indent);

    if new_indent != orig_indent {
        add_indent(result, new_indent - orig_indent);
    }
}

fn on_indent<'a>(result: &mut State<'a>) -> Result<()> {
    result.indent_x = Some(result.x);
    result.tracking_indent = false;

    if result.quote_danger {
        error(result, ErrorName::QuoteDanger)?;
    }

    match result.mode {
        Mode::Indent => {
            let x = result.x;
            correct_paren_trail(result, x);

            let to_add = match peek(&result.paren_stack, 0) {
                Some(opener) if should_add_opener_indent(result, opener) => Some(opener.indent_delta),
                _ => None
            };

            if let Some(adjust) = to_add {
                add_indent(result, adjust);
            }
        },
        Mode::Paren => correct_indent(result)
    }

    Ok(())
}

fn check_leading_close_paren<'a>(result: &mut State<'a>) -> Result<()> {
    if result.error_pos_cache.contains_key(&ErrorName::LeadingCloseParen) &&
      result.paren_trail.line_no == Some(result.line_no) {
        error(result, ErrorName::LeadingCloseParen)?;
    }

    Ok(())
}

fn on_leading_close_paren<'a>(result: &mut State<'a>) -> Result<()> {
    match result.mode {
        Mode::Indent => {
            if !result.force_balance {
                if result.smart {
                    error(result, ErrorName::LeadingCloseParen)?;
                }
                if !result.error_pos_cache.contains_key(&ErrorName::LeadingCloseParen) {
                    cache_error_pos(result, ErrorName::LeadingCloseParen);
                }
            }
            result.skip_char = true;
        },
        Mode::Paren => {
            if !is_valid_close_paren(&result.paren_stack, result.ch) {
                if result.smart {
                    result.skip_char = true;
                } else {
                    error(result, ErrorName::UnmatchedCloseParen)?;
                }
            } else if is_cursor_left_of(result.cursor_x, result.cursor_line, Some(result.x), result.line_no) {
                let line_no = result.line_no;
                let x = result.x;
                reset_paren_trail(result, line_no, x);
                on_indent(result)?;
            } else {
                append_paren_trail(result);
                result.skip_char = true;
            }
        }
    }

    Ok(())
}

fn on_comment_line<'a>(result: &mut State<'a>) {
    let paren_trail_length = result.paren_trail.openers.len();

    // restore the openers matching the previous paren trail
    if let Mode::Paren = result.mode {
        for j in 0..paren_trail_length {
            if let Some(opener) = peek(&result.paren_trail.openers, j) {
                result.paren_stack.push(Paren { ..*opener });
            }
        }
    };

    let x = result.x;
    let i = get_parent_opener_index(result, x);
    let mut indent_to_add : Delta = 0;
    if let Some(opener) = peek(&result.paren_stack, i) {
        // shift the comment line based on the parent open paren
        if should_add_opener_indent(result, opener) {
            indent_to_add = opener.indent_delta;
        }
        // TODO: store some information here if we need to place close-parens after comment lines
    }
    if indent_to_add != 0 {
        add_indent(result, indent_to_add);
    }

    // repop the openers matching the previous paren trail
    if let Mode::Paren = result.mode {
        for _ in 0..paren_trail_length {
            result.paren_stack.pop();
        }
    }
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

fn process_error<'a>(result: &mut State<'a>, e: Error) {
    result.success = false;
    result.error = Some(e);
}

fn process_text<'a>(text: &'a str, options: &Options<'a>, mode: Mode, smart: bool) -> Result<State<'a>> {
    let mut result = get_initial_result(text, &options, mode, smart);

    let mut process_result : Result<()> = Ok(());
    for i in 0..result.input_lines.len() {
        result.input_line_no = i;
        process_result = process_line(&mut result, i);
        if let Err(_) = process_result {
            break;
        }
    }
    if let Ok(_) = process_result {
        process_result = finalize_result(&mut result);
    }

    match process_result {
        Err(Error { name: ErrorName::Restart, .. }) => process_text(text, &options, Mode::Paren, smart),
        Err(e) => {
            process_error(&mut result, e);
            Ok(result)
        }
        _ => Ok(result)
    }
}

//------------------------------------------------------------------------------
// Public API
//------------------------------------------------------------------------------

fn public_result<'a>(result: State<'a>) -> State<'a> {
    unimplemented!();
}

pub fn indent_mode<'a>(text: &'a str, options: Options<'a>) -> Result<State<'a>> {
    process_text(text, &options, Mode::Indent, false).map(public_result)
}

pub fn paren_mode<'a>(text: &'a str, options: Options<'a>) -> Result<State<'a>> {
    process_text(text, &options, Mode::Paren, false).map(public_result)
}

pub fn smart_mode<'a>(text: &'a str, options: Options<'a>) -> Result<State<'a>> {
    let smart = options.selection_start_line == None;
    process_text(text, &options, Mode::Indent, smart).map(public_result)
}
