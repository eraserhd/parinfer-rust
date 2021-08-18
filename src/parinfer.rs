use super::std;
use std::collections::HashMap;
use std::borrow::Cow;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;
use types::*;
use changes;

// {{{1 Constants / Predicates

const BACKSLASH: &'static str = "\\";
const BLANK_SPACE: &'static str = " ";
const DOUBLE_SPACE: &'static str = "  ";
const VERTICAL_LINE: &'static str = "|";
const BANG: &'static str = "!";
const NUMBER_SIGN: &'static str = "#";
const NEWLINE: &'static str = "\n";
const TAB: &'static str = "\t";
const GRAVE: &'static str = "`";

fn match_paren(paren: &str) -> Option<&'static str> {
    match paren {
        "{" => Some("}"),
        "}" => Some("{"),
        "[" => Some("]"),
        "]" => Some("["),
        "(" => Some(")"),
        ")" => Some("("),
        _ => None,
    }
}

#[cfg(test)]
#[test]
fn match_paren_works() {
    assert_eq!(match_paren("}"), Some("{"));
    assert_eq!(match_paren("x"), None);
}

// {{{1 Options Structure

struct TransformedChange {
    old_end_x: Column,
    new_end_x: Column,
    lookup_line_no: LineNumber,
    lookup_x: Column,
}

pub fn chomp_cr<'a>(text: &'a str) -> &'a str {
    if text.chars().last() == Some('\r') {
        &text[0..text.len() - 1]
    } else {
        text
    }
}

fn split_lines<'a>(text: &'a str) -> Vec<&'a str> {
    text.split('\n').map(chomp_cr).collect()
}

fn transform_change<'a>(change: &'a Change) -> TransformedChange {
    let new_lines: Vec<&'a str> = change.new_text.split('\n').map(chomp_cr).collect();
    let old_lines: Vec<&'a str> = change.old_text.split('\n').map(chomp_cr).collect();

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

    let last_old_line_len = UnicodeWidthStr::width(old_lines[old_lines.len() - 1]);
    let last_new_line_len = UnicodeWidthStr::width(new_lines[new_lines.len() - 1]);

    let old_end_x = (if old_lines.len() == 1 { change.x } else { 0 }) + last_old_line_len;
    let new_end_x = (if new_lines.len() == 1 { change.x } else { 0 }) + last_new_line_len;
    let new_end_line_no = change.line_no + (new_lines.len() - 1);

    TransformedChange {
        old_end_x,
        new_end_x,

        lookup_line_no: new_end_line_no,
        lookup_x: new_end_x,
    }
}

fn transform_changes<'a>(
    changes: &Vec<Change>,
) -> HashMap<(LineNumber, Column), TransformedChange> {
    let mut lines: HashMap<(LineNumber, Column), TransformedChange> = HashMap::new();
    for change in changes {
        let transformed_change = transform_change(change);
        lines.insert(
            (
                transformed_change.lookup_line_no,
                transformed_change.lookup_x,
            ),
            transformed_change,
        );
    }
    lines
}

// {{{1 State Structure (was Result)

#[derive(Debug)]
struct ParenTrailClamped<'a> {
    start_x: Option<Column>,
    end_x: Option<Column>,
    openers: Vec<Paren<'a>>,
}

#[derive(Debug)]
struct InternalParenTrail<'a> {
    line_no: Option<LineNumber>,
    start_x: Option<Column>,
    end_x: Option<Column>,
    openers: Vec<Paren<'a>>,
    clamped: ParenTrailClamped<'a>,
}

#[derive(PartialEq, Eq)]
pub enum Mode {
    Indent,
    Paren,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum TrackingArgTabStop {
    NotSearching,
    Space,
    Arg,
}

#[derive(PartialEq, Eq)]
enum Now {
    Normal,
    Escaping,
    Escaped,
}

impl<'a> State<'a> {
    fn is_escaping(&self) -> bool {
        match self.escape { Now::Escaping => true, _ => false }
    }
    fn is_escaped(&self) -> bool {
        match self.escape { Now::Escaped => true, _ => false }
    }
}

#[derive(PartialEq, Eq)]
enum In<'a> {
    Code,
    Comment,
    String { delim: &'a str },
    LispReaderSyntax,
    LispBlockCommentPre { depth: usize },
    LispBlockComment { depth: usize },
    LispBlockCommentPost { depth: usize },
    GuileBlockComment,
    GuileBlockCommentPost,
    JanetLongStringPre { open_delim_len: usize },
    JanetLongString { open_delim_len: usize, close_delim_len: usize },
}

impl<'a> State<'a> {
    fn is_in_code(&self) -> bool {
        match self.context {
            In::Code => true,
            In::LispReaderSyntax => true,
            _ => false
        }
    }
    fn is_in_comment(&self) -> bool {
        match self.context { In::Comment => true, _ => false }
    }
    fn is_in_stringish(&self) -> bool {
        match self.context {
            In::String {..} => true,
            In::LispBlockCommentPre {..} => true,
            In::LispBlockComment {..} => true,
            In::LispBlockCommentPost {..} => true,
            In::GuileBlockComment => true,
            In::GuileBlockCommentPost => true,
            In::JanetLongStringPre {..} => true,
            In::JanetLongString {..} => true,
            _ => false
        }
    }
}

struct State<'a> {
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

    tab_stops: Vec<TabStop<'a>>,

    paren_trail: InternalParenTrail<'a>,
    paren_trails: Vec<ParenTrail>,

    return_parens: bool,
    parens: Vec<Paren<'a>>,

    cursor_x: Option<Column>,
    cursor_line: Option<LineNumber>,
    prev_cursor_x: Option<Column>,
    prev_cursor_line: Option<Column>,

    selection_start_line: Option<LineNumber>,

    changes: HashMap<(LineNumber, Column), TransformedChange>,

    context: In<'a>,
    comment_x: Option<Column>,
    escape: Now,

    lisp_vline_symbols_enabled: bool,
    lisp_reader_syntax_enabled: bool,
    lisp_block_comments_enabled: bool,
    guile_block_comments_enabled: bool,
    scheme_sexp_comments_enabled: bool,
    janet_long_strings_enabled: bool,

    quote_danger: bool,
    tracking_indent: bool,
    skip_char: bool,
    success: bool,
    partial_result: bool,
    force_balance: bool,

    comment_char: String,
    string_delimiters: Vec<String>,

    max_indent: Option<Column>,
    indent_delta: i64,

    tracking_arg_tab_stop: TrackingArgTabStop,

    error: Option<Error>,
    error_pos_cache: HashMap<ErrorName, Error>,
}

fn initial_paren_trail<'a>() -> InternalParenTrail<'a> {
    InternalParenTrail {
        line_no: None,
        start_x: None,
        end_x: None,
        openers: vec![],
        clamped: ParenTrailClamped {
            start_x: None,
            end_x: None,
            openers: vec![],
        },
    }
}

fn get_initial_result<'a>(
    text: &'a str,
    options: &Options,
    mode: Mode,
    smart: bool,
) -> State<'a> {
    let lisp_reader_syntax_enabled = [
        options.lisp_block_comments,
        options.guile_block_comments,
        options.scheme_sexp_comments,
    ].iter().any(|is_true| *is_true);

    State {
        mode: mode,
        smart: smart,

        orig_text: text,
        orig_cursor_x: options.cursor_x,
        orig_cursor_line: options.cursor_line,

        input_lines: split_lines(text),
        input_line_no: 0,
        input_x: 0,

        lines: vec![],
        line_no: usize::max_value(),
        ch: &text[0..0],
        x: 0,
        indent_x: None,

        paren_stack: vec![],
        tab_stops: vec![],

        paren_trail: initial_paren_trail(),
        paren_trails: vec![],

        return_parens: false,
        parens: vec![],

        cursor_x: options.cursor_x,
        cursor_line: options.cursor_line,
        prev_cursor_x: options.prev_cursor_x,
        prev_cursor_line: options.prev_cursor_line,

        selection_start_line: None,

        changes: transform_changes(&options.changes),

        context: In::Code,
        comment_x: None,
        escape: Now::Normal,

        lisp_vline_symbols_enabled: options.lisp_vline_symbols,
        lisp_reader_syntax_enabled,
        lisp_block_comments_enabled: options.lisp_block_comments,
        guile_block_comments_enabled: options.guile_block_comments,
        scheme_sexp_comments_enabled: options.scheme_sexp_comments,
        janet_long_strings_enabled: options.janet_long_strings,

        quote_danger: false,
        tracking_indent: false,
        skip_char: false,
        success: false,
        partial_result: false,
        force_balance: false,

        comment_char: options.comment_char.to_string(),
        string_delimiters: options.string_delimiters.clone(),

        max_indent: None,
        indent_delta: 0,

        tracking_arg_tab_stop: TrackingArgTabStop::NotSearching,

        error: None,
        error_pos_cache: HashMap::new(),
    }
}

// {{{1 Possible Errors

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
        ErrorName::Utf8EncodingError => "UTF8 encoded incorrectly.",
        ErrorName::JsonEncodingError => "JSON encoded incorrectly.",
        ErrorName::Panic => "Internal error (please report!)",

        ErrorName::Restart => "Restart requested (you shouldn't see this).",
    }
}

fn cache_error_pos(result: &mut State, name: ErrorName) {
    let error = Error {
        name,
        message: String::new(),
        line_no: result.line_no,
        x: result.x,
        input_line_no: result.input_line_no,
        input_x: result.input_x,
    };
    result.error_pos_cache.insert(name, error);
}

fn error(result: &mut State, name: ErrorName) -> Result<()> {
    let (line_no, x) = match (result.partial_result, result.error_pos_cache.get(&name)) {
        (true, Some(cache)) => (cache.line_no, cache.x),
        (false, Some(cache)) => (cache.input_line_no, cache.input_x),
        (true, None) => (result.line_no, result.x),
        (false, None) => (result.input_line_no, result.input_x),
    };

    let mut e = Error {
        name,
        line_no,
        x,
        message: String::from(error_message(name)),
        input_line_no: result.input_line_no,
        input_x: result.input_x,
    };

    if name == ErrorName::UnclosedParen {
        if let Some(opener) = peek(&result.paren_stack, 0) {
            e.line_no = if result.partial_result {
                opener.line_no
            } else {
                opener.input_line_no
            };
            e.x = if result.partial_result {
                opener.x
            } else {
                opener.input_x
            };
        }
    }

    Err(e)
}

// {{{1 String Operations

fn column_byte_index(s: &str, x: usize) -> usize {
    s.grapheme_indices(true)
        .scan(0, |column, (idx, ch)| {
            let start_column = *column;
            *column = *column + UnicodeWidthStr::width(ch);
            Some((start_column, (idx, ch)))
        })
        .filter_map(|(n, (idx, _))| if n == x { Some(idx) } else { None })
        .nth(0) 
        .unwrap_or_else(|| s.len())
}

#[cfg(test)]
#[test]
fn column_byte_index_works() {
    assert_eq!(column_byte_index("abc", 1), 1);
    assert_eq!(column_byte_index("abc", 3), 3);
    assert_eq!(column_byte_index("åbc", 3), 4);
    assert_eq!(column_byte_index("åbc", 1), 2);
    assert_eq!(column_byte_index("ｗｏ", 4), 6);
    assert_eq!(column_byte_index("ｗｏ", 2), 3);
    assert_eq!(column_byte_index("ｗｏ", 0), 0);
}

fn replace_within_string(orig: &str, start: usize, end: usize, replace: &str) -> String {
    let start_i = column_byte_index(orig, start);
    let end_i = column_byte_index(orig, end);
    String::from(&orig[0..start_i]) + replace + &orig[end_i..]
}

#[cfg(test)]
#[test]
fn replace_within_string_works() {
    assert_eq!(replace_within_string("aaa", 0, 2, ""), "a");
    assert_eq!(replace_within_string("aaa", 0, 1, "b"), "baa");
    assert_eq!(replace_within_string("aaa", 0, 2, "b"), "ba");
    assert_eq!(replace_within_string("ééé", 0, 2, ""), "é");
    assert_eq!(replace_within_string("ééé", 0, 1, "b"), "béé");
    assert_eq!(replace_within_string("ééé", 1, 2, "b"), "ébé");
    assert_eq!(replace_within_string("ééé", 0, 2, "b"), "bé");
    assert_eq!(replace_within_string("ééé", 3, 3, "b"), "éééb");
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

// {{{1 Line operations

fn is_cursor_affected<'a>(result: &State<'a>, start: Column, end: Column) -> bool {
    match result.cursor_x {
        Some(x) if x == start && x == end => x == 0,
        Some(x) => x >= end,
        None => false,
    }
}

fn shift_cursor_on_edit<'a>(
    result: &mut State<'a>,
    line_no: LineNumber,
    start: Column,
    end: Column,
    replace: &str,
) {
    let old_length = end - start;
    let new_length = UnicodeWidthStr::width(replace);
    let dx = new_length as Delta - old_length as Delta;

    if let (Some(cursor_x), Some(cursor_line)) = (result.cursor_x, result.cursor_line) {
        if dx != 0 && cursor_line == line_no && is_cursor_affected(result, start, end) {
            result.cursor_x = Some(((cursor_x as Delta) + dx) as usize);
        }
    }
}

fn replace_within_line<'a>(
    result: &mut State<'a>,
    line_no: LineNumber,
    start: Column,
    end: Column,
    replace: &str,
) {
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
    result.line_no = usize::wrapping_add(result.line_no, 1);

    // reset line-specific state
    result.indent_x = None;
    result.comment_x = None;
    result.indent_delta = 0;

    result
        .error_pos_cache
        .remove(&ErrorName::UnmatchedCloseParen);
    result
        .error_pos_cache
        .remove(&ErrorName::UnmatchedOpenParen);
    result.error_pos_cache.remove(&ErrorName::LeadingCloseParen);

    result.tracking_arg_tab_stop = TrackingArgTabStop::NotSearching;
    result.tracking_indent = !result.is_in_stringish();
}

fn commit_char<'a>(result: &mut State<'a>, orig_ch: &'a str) {
    let ch = result.ch;
    let ch_width = UnicodeWidthStr::width(ch);
    if orig_ch != ch {
        let line_no = result.line_no;
        let x = result.x;
        let orig_ch_width = UnicodeWidthStr::width(orig_ch);
        replace_within_line(result, line_no, x, x + orig_ch_width, ch);
        result.indent_delta -= orig_ch_width as Delta - ch_width as Delta;
    }
    result.x += ch_width;
}

// {{{1 Misc Utils

fn clamp<T: Clone + Ord>(val: T, min_n: Option<T>, max_n: Option<T>) -> T {
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
    let empty: Vec<char> = vec![];
    assert_eq!(peek(&empty, 0), None);
    assert_eq!(peek(&empty, 1), None);
}

// {{{1 Questions about characters

fn is_close_paren(paren: &str) -> bool {
    match paren {
        "}" | "]" | ")" => true,
        _ => false,
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
    !result.is_escaped() && (result.ch == BLANK_SPACE || result.ch == DOUBLE_SPACE)
}

fn is_closable<'a>(result: &State<'a>) -> bool {
    let ch = result.ch;
    let closer = is_close_paren(ch) && !result.is_escaped();
    return result.is_in_code() && !is_whitespace(result) && ch != "" && !closer;
}


// {{{1 Advanced operations on characters

fn check_cursor_holding<'a>(result: &State<'a>) -> Result<bool> {
    let opener = peek(&result.paren_stack, 0).unwrap();
    let hold_min_x = peek(&result.paren_stack, 1).map(|p| p.x + 1).unwrap_or(0);
    let hold_max_x = opener.x;

    let holding = result.cursor_line == Some(opener.line_no)
        && result.cursor_x.map(|x| hold_min_x <= x).unwrap_or(false)
        && result.cursor_x.map(|x| x <= hold_max_x).unwrap_or(false);
    let should_check_prev = result.changes.is_empty() && result.prev_cursor_line != None;
    if should_check_prev {
        let prev_holding = result.prev_cursor_line == Some(opener.line_no)
            && result
                .prev_cursor_x
                .map(|x| hold_min_x <= x)
                .unwrap_or(false)
            && result
                .prev_cursor_x
                .map(|x| x <= hold_max_x)
                .unwrap_or(false);
        if prev_holding && !holding {
            return Err(Error {
                name: ErrorName::Restart,
                x: 0,
                input_line_no: 0,
                input_x: 0,
                line_no: 0,
                message: String::new(),
            });
        }
    }

    Ok(holding)
}

fn track_arg_tab_stop<'a>(result: &mut State<'a>, state: TrackingArgTabStop) {
    if state == TrackingArgTabStop::Space {
        if result.is_in_code() && is_whitespace(result) {
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

// {{{1 Literal character events

fn in_code_on_open_paren<'a>(result: &mut State<'a>) {
    let opener = Paren {
        input_line_no: result.input_line_no,
        input_x: result.input_x,

        line_no: result.line_no,
        x: result.x,
        ch: result.ch,
        indent_delta: result.indent_delta,
        max_child_indent: None,

        arg_x: None,

        closer: None,
        children: vec![]
    };

    if result.return_parens {
        if let Some(parent) = result.paren_stack.last_mut() {
            parent.children.push(opener.clone());
        } else {
            result.parens.push(opener.clone());
        }
    }
    result.paren_stack.push(opener);
    result.tracking_arg_tab_stop = TrackingArgTabStop::Space;
}

fn in_code_on_matched_close_paren<'a>(result: &mut State<'a>) -> Result<()> {
    let mut opener = (*peek(&result.paren_stack, 0).unwrap()).clone();
    if result.return_parens {
        set_closer(&mut opener, result.line_no, result.x, result.ch);
    }

    result.paren_trail.end_x = Some(result.x + 1);
    result.paren_trail.openers.push(opener);

    if result.mode == Mode::Indent && result.smart && check_cursor_holding(result)? {
        let orig_start_x = result.paren_trail.start_x;
        let orig_end_x = result.paren_trail.end_x;
        let orig_openers = result.paren_trail.openers.clone();
        let x = result.x;
        let line_no = result.line_no;
        reset_paren_trail(result, line_no, x + 1);
        result.paren_trail.clamped = ParenTrailClamped {
            start_x: orig_start_x,
            end_x: orig_end_x,
            openers: orig_openers,
        };
    }
    result.paren_stack.pop();
    result.tracking_arg_tab_stop = TrackingArgTabStop::NotSearching;

    Ok(())
}

fn in_code_on_unmatched_close_paren<'a>(result: &mut State<'a>) -> Result<()> {
    match result.mode {
        Mode::Paren => {
            let in_leading_paren_trail = result.paren_trail.line_no == Some(result.line_no)
                && result.paren_trail.start_x == result.indent_x;
            let can_remove = result.smart && in_leading_paren_trail;
            if !can_remove {
                error(result, ErrorName::UnmatchedCloseParen)?;
            }
        }
        Mode::Indent => {
            if !result
                .error_pos_cache
                .contains_key(&ErrorName::UnmatchedCloseParen)
            {
                cache_error_pos(result, ErrorName::UnmatchedCloseParen);
                if peek(&result.paren_stack, 0).is_some() {
                    cache_error_pos(result, ErrorName::UnmatchedOpenParen);
                    let opener = peek(&result.paren_stack, 0).unwrap();
                    if let Some(err) = result
                        .error_pos_cache
                        .get_mut(&ErrorName::UnmatchedOpenParen)
                    {
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

fn in_code_on_close_paren<'a>(result: &mut State<'a>) -> Result<()> {
    if is_valid_close_paren(&result.paren_stack, result.ch) {
        in_code_on_matched_close_paren(result)?;
    } else {
        in_code_on_unmatched_close_paren(result)?;
    }

    Ok(())
}

fn in_code_on_tab<'a>(result: &mut State<'a>) {
    result.ch = DOUBLE_SPACE;
}

fn in_code_on_comment_char<'a>(result: &mut State<'a>) {
    result.context = In::Comment;
    result.comment_x = Some(result.x);
    result.tracking_arg_tab_stop = TrackingArgTabStop::NotSearching;
}

fn on_newline<'a>(result: &mut State<'a>) {
    if result.is_in_comment() {
        result.context = In::Code;
    }
    result.ch = "";
}

fn in_code_on_quote<'a>(result: &mut State<'a>) {
    result.context = In::String { delim: result.ch };
    cache_error_pos(result, ErrorName::UnclosedQuote);
}
fn in_comment_on_quote<'a>(result: &mut State<'a>) {
    result.quote_danger = !result.quote_danger;
    if result.quote_danger {
        cache_error_pos(result, ErrorName::QuoteDanger);
    }
}
fn in_string_on_quote<'a>(result: &mut State<'a>, delim: &'a str) {
    if delim == result.ch {
        result.context = In::Code;
    }
}

fn in_code_on_nsign<'a>(result: &mut State<'a>) {
    result.context = In::LispReaderSyntax;
}

fn in_lisp_reader_syntax_on_vline<'a>(result: &mut State<'a>) {
    result.context = In::LispBlockComment { depth: 1 };
}
fn in_lisp_reader_syntax_on_bang<'a>(result: &mut State<'a>) {
    result.context = In::GuileBlockComment;
}
fn in_lisp_reader_syntax_on_semicolon<'a>(result: &mut State<'a>) {
    result.context = In::Code;
}

fn in_lisp_block_comment_pre_on_vline<'a>(result: &mut State<'a>, depth: usize) {
    result.context = In::LispBlockComment { depth: depth + 1 };
}
fn in_lisp_block_comment_pre_on_else<'a>(result: &mut State<'a>, depth: usize) {
    result.context = In::LispBlockComment { depth };
}
fn in_lisp_block_comment_on_nsign<'a>(result: &mut State<'a>, depth: usize) {
    result.context = In::LispBlockCommentPre { depth };
}
fn in_lisp_block_comment_on_vline<'a>(result: &mut State<'a>, depth: usize) {
    result.context = In::LispBlockCommentPost { depth };
}
fn in_lisp_block_comment_post_on_nsign<'a>(result: &mut State<'a>, depth: usize) {
    let depth = depth - 1;
    if depth > 0 {
        result.context = In::LispBlockComment { depth };
    } else {
        result.context = In::Code;
    }
}
fn in_lisp_block_comment_post_on_else<'a>(result: &mut State<'a>, depth: usize) {
    result.context = In::LispBlockComment { depth };
}

fn in_guile_block_comment_on_bang<'a>(result: &mut State<'a>) {
    result.context = In::GuileBlockCommentPost;
}
fn in_guile_block_comment_post_on_nsign<'a>(result: &mut State<'a>) {
    result.context = In::Code;
}
fn in_guile_block_comment_post_on_else<'a>(result: &mut State<'a>) {
    result.context = In::GuileBlockComment;
}

fn in_code_on_grave<'a>(result: &mut State<'a>) {
    result.context = In::JanetLongStringPre { open_delim_len: 1 };
    cache_error_pos(result, ErrorName::UnclosedQuote);
}
fn in_janet_long_string_pre_on_grave<'a>(result: &mut State<'a>, open_delim_len: usize) {
    result.context = In::JanetLongStringPre { open_delim_len: open_delim_len + 1 };
}
fn in_janet_long_string_pre_on_else<'a>(result: &mut State<'a>, open_delim_len: usize) {
    result.context = In::JanetLongString { open_delim_len, close_delim_len: 0 };
}
fn in_janet_long_string_on_grave<'a>(result: &mut State<'a>, open_delim_len: usize, close_delim_len: usize) {
    let close_delim_len = close_delim_len + 1;
    if open_delim_len == close_delim_len {
        result.context = In::Code;
    } else {
        result.context = In::JanetLongString { open_delim_len, close_delim_len };
    }
}
fn in_janet_long_string_on_else<'a>(result: &mut State<'a>, open_delim_len: usize, close_delim_len: usize) {
    if close_delim_len > 0 {
        result.context = In::JanetLongString { open_delim_len, close_delim_len: 0 };
    }
}

fn on_backslash<'a>(result: &mut State<'a>) {
    result.escape = Now::Escaping;
}

fn after_backslash<'a>(result: &mut State<'a>) -> Result<()> {
    result.escape = Now::Escaped;

    if result.ch == NEWLINE {
        if result.is_in_code() {
            return error(result, ErrorName::EolBackslash);
        }
    }

    Ok(())
}

// {{{1 Character dispatch

fn on_context<'a>(result: &mut State<'a>) -> Result<()> {
    let ch = result.ch;
    match result.context {
        In::Code => {
            match ch {
                _ if ch == result.comment_char => in_code_on_comment_char(result),
                _ if result.string_delimiters.contains(&ch.to_string()) => in_code_on_quote(result),
                "(" | "[" | "{" => in_code_on_open_paren(result),
                ")" | "]" | "}" => in_code_on_close_paren(result)?,
                VERTICAL_LINE if result.lisp_vline_symbols_enabled => in_code_on_quote(result),
                NUMBER_SIGN if result.lisp_reader_syntax_enabled => in_code_on_nsign(result),
                GRAVE if result.janet_long_strings_enabled => in_code_on_grave(result),
                TAB => in_code_on_tab(result),
                _ => (),
            }
        },
        In::Comment => {
            match ch {
                _ if result.string_delimiters.contains(&ch.to_string()) => in_comment_on_quote(result),
                VERTICAL_LINE if result.lisp_vline_symbols_enabled => in_comment_on_quote(result),
                GRAVE if result.janet_long_strings_enabled => in_comment_on_quote(result),
                _ => (),
            }
        },
        In::String { delim } => {
            match ch {
                _ if result.string_delimiters.contains(&ch.to_string()) => in_string_on_quote(result, delim),
                VERTICAL_LINE if result.lisp_vline_symbols_enabled => in_string_on_quote(result, delim),
                _ => (),
            }
        },
        In::LispReaderSyntax => {
            match ch {
                VERTICAL_LINE if result.lisp_block_comments_enabled => in_lisp_reader_syntax_on_vline(result),
                BANG if result.guile_block_comments_enabled => in_lisp_reader_syntax_on_bang(result),
                ";" if result.scheme_sexp_comments_enabled => in_lisp_reader_syntax_on_semicolon(result),
                _ => {
                    // Backtrack!
                    result.context = In::Code;
                    on_context(result)?
                },
            }
        },
        In::LispBlockCommentPre { depth } => {
            match ch {
                VERTICAL_LINE => in_lisp_block_comment_pre_on_vline(result, depth),
                _ => in_lisp_block_comment_pre_on_else(result, depth),
            }
        },
        In::LispBlockComment { depth } => {
            match ch {
                NUMBER_SIGN => in_lisp_block_comment_on_nsign(result, depth),
                VERTICAL_LINE => in_lisp_block_comment_on_vline(result, depth),
                _ => (),
            }
        },
        In::LispBlockCommentPost { depth } => {
            match ch {
                NUMBER_SIGN => in_lisp_block_comment_post_on_nsign(result, depth),
                _ => in_lisp_block_comment_post_on_else(result, depth),
            }
        },
        In::GuileBlockComment => {
            match ch {
                BANG => in_guile_block_comment_on_bang(result),
                _ => (),
            }
        },
        In::GuileBlockCommentPost => {
            match ch {
                NUMBER_SIGN => in_guile_block_comment_post_on_nsign(result),
                _ => in_guile_block_comment_post_on_else(result),
            }
        },
        In::JanetLongStringPre { open_delim_len } => {
            match ch {
                GRAVE => in_janet_long_string_pre_on_grave(result, open_delim_len),
                _ => in_janet_long_string_pre_on_else(result, open_delim_len),
            }
        },
        In::JanetLongString { open_delim_len, close_delim_len } => {
            match ch {
                GRAVE => in_janet_long_string_on_grave(result, open_delim_len, close_delim_len),
                _ => in_janet_long_string_on_else(result, open_delim_len, close_delim_len),
            }
        },
    }

    Ok(())
}

fn on_char<'a>(result: &mut State<'a>) -> Result<()> {
    let mut ch = result.ch;
    if result.is_escaped() {
        result.escape = Now::Normal;
    }

    if result.is_escaping() {
        after_backslash(result)?;
    } else if ch == BACKSLASH {
        on_backslash(result);
    } else if ch == NEWLINE {
        on_newline(result);
    } else {
        on_context(result)?;
    }

    ch = result.ch;

    if is_closable(result) {
        let line_no = result.line_no;
        let x = result.x;
        reset_paren_trail(result, line_no, x + UnicodeWidthStr::width(ch));
    }

    let state = result.tracking_arg_tab_stop;
    if state != TrackingArgTabStop::NotSearching {
        track_arg_tab_stop(result, state);
    }

    Ok(())
}

// {{{1 Cursor functions

fn is_cursor_left_of<'a>(
    cursor_x: Option<Column>,
    cursor_line: Option<LineNumber>,
    x: Option<Column>,
    line_no: LineNumber,
) -> bool {
    if let (Some(x), Some(cursor_x)) = (x, cursor_x) {
        cursor_line == Some(line_no) && cursor_x <= x // inclusive since (cursorX = x) implies (x-1 < cursor < x)
    } else {
        false
    }
}

fn is_cursor_right_of<'a>(
    cursor_x: Option<Column>,
    cursor_line: Option<LineNumber>,
    x: Option<Column>,
    line_no: LineNumber,
) -> bool {
    if let (Some(x), Some(cursor_x)) = (x, cursor_x) {
        cursor_line == Some(line_no) && cursor_x > x
    } else {
        false
    }
}

fn is_cursor_in_comment<'a>(
    result: &State<'a>,
    cursor_x: Option<Column>,
    cursor_line: Option<LineNumber>,
) -> bool {
    is_cursor_right_of(cursor_x, cursor_line, result.comment_x, result.line_no)
}

fn handle_change_delta<'a>(result: &mut State<'a>) {
    if !result.changes.is_empty() && (result.smart || result.mode == Mode::Paren) {
        if let Some(change) = result.changes.get(&(result.input_line_no, result.input_x)) {
            result.indent_delta += change.new_end_x as Delta - change.old_end_x as Delta;
        }
    }
}

// {{{1 Paren Trail functions

fn reset_paren_trail<'a>(result: &mut State<'a>, line_no: LineNumber, x: Column) {
    result.paren_trail.line_no = Some(line_no);
    result.paren_trail.start_x = Some(x);
    result.paren_trail.end_x = Some(x);
    result.paren_trail.openers = vec![];
    result.paren_trail.clamped.start_x = None;
    result.paren_trail.clamped.end_x = None;
    result.paren_trail.clamped.openers = vec![];
}

fn is_cursor_clamping_paren_trail<'a>(
    result: &State<'a>,
    cursor_x: Option<Column>,
    cursor_line: Option<LineNumber>,
) -> bool {
    is_cursor_right_of(
        cursor_x,
        cursor_line,
        result.paren_trail.start_x,
        result.line_no,
    ) && !is_cursor_in_comment(result, cursor_x, cursor_line)
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
        for (x, ch) in line
            .graphemes(true)
            .scan(0, |column, ch| {
                let start_column = *column;
                *column = *column + UnicodeWidthStr::width(ch);
                Some((start_column, ch))
            })
        {
            if x < start_x || x >= new_start_x {
                continue;
            }
            if is_close_paren(ch) {
                remove_count += 1;
            }
        }

        let openers = result.paren_trail.openers.clone();

        result.paren_trail.openers = (&openers[remove_count..]).to_vec();
        result.paren_trail.start_x = Some(new_start_x);
        result.paren_trail.end_x = Some(new_end_x);

        result.paren_trail.clamped.openers = (&openers[..remove_count]).to_vec();
        result.paren_trail.clamped.start_x = Some(start_x);
        result.paren_trail.clamped.end_x = Some(end_x);
    }
}

fn pop_paren_trail<'a>(result: &mut State<'a>) {
    let start_x = result.paren_trail.start_x;
    let end_x = result.paren_trail.end_x;

    if start_x == end_x {
        return;
    }

    while let Some(paren) = result.paren_trail.openers.pop() {
        result.paren_stack.push(paren);
    }
}

fn get_parent_opener_index<'a>(result: &mut State<'a>, indent_x: usize) -> usize {
    for i in 0..result.paren_stack.len() {
        let opener = peek(&result.paren_stack, i).unwrap().clone();
        let opener_index = result.paren_stack.len() - i - 1;

        let curr_outside = opener.x < indent_x;

        let prev_indent_x = indent_x as Delta - result.indent_delta;
        let prev_outside = opener.x as Delta - opener.indent_delta < prev_indent_x;

        let mut is_parent = false;

        if prev_outside && curr_outside {
            is_parent = true;
        } else if !prev_outside && !curr_outside {
            is_parent = false;
        } else if prev_outside && !curr_outside {
            // POSSIBLE FRAGMENTATION
            // (foo    --\
            //            +--- FRAGMENT `(foo bar)` => `(foo) bar`
            // bar)    --/

            // 1. PREVENT FRAGMENTATION
            // ```in
            //   (foo
            // ++
            //   bar
            // ```
            // ```out
            //   (foo
            //     bar
            // ```
            if result.indent_delta == 0 {
                is_parent = true;
            }
            // 2. ALLOW FRAGMENTATION
            // ```in
            // (foo
            //   bar
            // --
            // ```
            // ```out
            // (foo)
            // bar
            // ```
            else if opener.indent_delta == 0 {
                is_parent = false;
            } else {
                // TODO: identify legitimate cases where both are nonzero

                // allow the fragmentation by default
                is_parent = false;

                // TODO: should we throw to exit instead?  either of:
                // 1. give up, just `throw error(...)`
                // 2. fallback to paren mode to preserve structure
            }
        } else if !prev_outside && curr_outside {
            // POSSIBLE ADOPTION
            // (foo)   --\
            //            +--- ADOPT `(foo) bar` => `(foo bar)`
            //   bar   --/

            {
                let next_opener = peek(&result.paren_stack, i + 1);

                // 1. DISALLOW ADOPTION
                // ```in
                //   (foo
                // --
                //     (bar)
                // --
                //     baz)
                // ```
                // ```out
                // (foo
                //   (bar)
                //   baz)
                // ```
                // OR
                // ```in
                //   (foo
                // --
                //     (bar)
                // -
                //     baz)
                // ```
                // ```out
                // (foo
                //  (bar)
                //  baz)
                // ```
                if next_opener
                    .map(|no| no.indent_delta <= opener.indent_delta)
                    .unwrap_or(false)
                {
                    // we can only disallow adoption if nextOpener.indentDelta will actually
                    // prevent the indentX from being in the opener's threshold.
                    if indent_x as Delta + next_opener.unwrap().indent_delta > opener.x as Delta {
                        is_parent = true;
                    } else {
                        is_parent = false;
                    }
                }
                // 2. ALLOW ADOPTION
                // ```in
                // (foo
                //     (bar)
                // --
                //     baz)
                // ```
                // ```out
                // (foo
                //   (bar
                //     baz))
                // ```
                // OR
                // ```in
                //   (foo
                // -
                //     (bar)
                // --
                //     baz)
                // ```
                // ```out
                //  (foo
                //   (bar)
                //    baz)
                // ```
                else if next_opener
                    .map(|no| no.indent_delta > opener.indent_delta)
                    .unwrap_or(false)
                {
                    is_parent = true;
                }
                // 3. ALLOW ADOPTION
                // ```in
                //   (foo)
                // --
                //   bar
                // ```
                // ```out
                // (foo
                //   bar)
                // ```
                // OR
                // ```in
                // (foo)
                //   bar
                // ++
                // ```
                // ```out
                // (foo
                //   bar
                // ```
                // OR
                // ```in
                //  (foo)
                // +
                //   bar
                // ++
                // ```
                // ```out
                //  (foo
                //   bar)
                // ```
                else if result.indent_delta > opener.indent_delta {
                    is_parent = true;
                }
            }

            if is_parent {
                // if new parent
                // Clear `indentDelta` since it is reserved for previous child lines only.
                result.paren_stack[opener_index].indent_delta = 0;
            }
        }

        if is_parent {
            return i;
        }
    }

    result.paren_stack.len()
}

// INDENT MODE: correct paren trail from indentation
fn correct_paren_trail<'a>(result: &mut State<'a>, indent_x: usize) {
    let mut parens = String::new();

    let index = get_parent_opener_index(result, indent_x);
    for i in 0..index {
        let mut opener = result.paren_stack.pop().unwrap();
        let close_ch = match_paren(opener.ch).unwrap();
        if result.return_parens {
            opener.closer = Some(Closer {
                line_no: result.paren_trail.line_no.unwrap(),
                x: result.paren_trail.start_x.unwrap() + i,
                ch: close_ch,
                trail: None
            });
        }
        result.paren_trail.openers.push(opener);
        parens.push_str(close_ch);

    }

    if let Some(line_no) = result.paren_trail.line_no {
        let start_x = result.paren_trail.start_x.unwrap();
        let end_x = result.paren_trail.end_x.unwrap();
        replace_within_line(result, line_no, start_x, end_x, &parens[..]);
        result.paren_trail.end_x = result.paren_trail.start_x.map(|x| x + parens.len());
        remember_paren_trail(result);
    }
}

fn clean_paren_trail<'a>(result: &mut State<'a>) {
    let start_x = result.paren_trail.start_x;
    let end_x = result.paren_trail.end_x;

    if start_x == end_x || Some(result.line_no) != result.paren_trail.line_no {
        return;
    }

    let start_x = start_x.unwrap();
    let end_x = end_x.unwrap();

    let mut new_trail = String::new();
    let mut space_count = 0;
    for (x, ch) in result.lines[result.line_no]
                    .graphemes(true)
                    .scan(0, |column, ch| {
                        let start_column = *column;
                        *column = *column + UnicodeWidthStr::width(ch);
                        Some((start_column, ch))
                    })
    {
        if x < start_x || x >= end_x {
            continue;
        }

        if is_close_paren(ch) {
            new_trail.push_str(ch);
        } else {
            space_count += 1;
        }
    }

    if space_count > 0 {
        let line_no = result.line_no;
        replace_within_line(result, line_no, start_x, end_x, &new_trail[..]);
        result.paren_trail.end_x = result.paren_trail.end_x.map(|x| x - space_count);
    }
}

fn set_closer<'a>(opener: &mut Paren<'a>, line_no: LineNumber, x: Column, ch: &'a str) {
    opener.closer = Some(Closer { line_no, x, ch, trail: None })
}

fn append_paren_trail<'a>(result: &mut State<'a>) {
    let mut opener = result.paren_stack.pop().unwrap().clone();
    let close_ch = match_paren(opener.ch).unwrap();
    if result.return_parens {
        set_closer(&mut opener, result.paren_trail.line_no.unwrap(), result.paren_trail.end_x.unwrap(), close_ch);
    }

    set_max_indent(result, &opener);
    let line_no = result.paren_trail.line_no.unwrap();
    let end_x = result.paren_trail.end_x.unwrap();
    insert_within_line(result, line_no, end_x, close_ch);

    result.paren_trail.end_x = result.paren_trail.end_x.map(|x| x + 1);
    result.paren_trail.openers.push(opener);
    update_remembered_paren_trail(result);
}

fn invalidate_paren_trail<'a>(result: &mut State<'a>) {
    result.paren_trail = initial_paren_trail();
}

fn check_unmatched_outside_paren_trail<'a>(result: &mut State<'a>) -> Result<()> {
    let mut do_error = false;
    if let Some(cache) = result.error_pos_cache.get(&ErrorName::UnmatchedCloseParen) {
        if result
            .paren_trail
            .start_x
            .map(|x| cache.x < x)
            .unwrap_or(false)
        {
            do_error = true;
        }
    }

    if do_error {
        error(result, ErrorName::UnmatchedCloseParen)?;
    }

    Ok(())
}

fn set_max_indent<'a>(result: &mut State<'a>, opener: &Paren<'a>) {
    if let Some(parent) = result.paren_stack.last_mut() {
        parent.max_child_indent = Some(opener.x);
    } else {
        result.max_indent = Some(opener.x);
    }
}

fn remember_paren_trail<'a>(result: &mut State<'a>) {
    if result.paren_trail.clamped.openers.len() > 0 || result.paren_trail.openers.len() > 0 {
        let is_clamped = result.paren_trail.clamped.start_x != None;
        let short_trail = ParenTrail {
            line_no: result.paren_trail.line_no.unwrap(),
            start_x: if is_clamped {
                result.paren_trail.clamped.start_x
            } else {
                result.paren_trail.start_x
            }.unwrap(),
            end_x: if is_clamped {
                result.paren_trail.clamped.end_x
            } else {
                result.paren_trail.end_x
            }.unwrap(),
        };

        result.paren_trails.push(short_trail.clone());

        if result.return_parens {
            for opener in result.paren_trail.openers.iter_mut() {
                opener.closer.as_mut().unwrap().trail = Some(short_trail.clone());
            }
        }
    }
}

fn update_remembered_paren_trail<'a>(result: &mut State<'a>) {
    if result.paren_trails.is_empty()
        || Some(result.paren_trails[result.paren_trails.len() - 1].line_no)
            != result.paren_trail.line_no
    {
        remember_paren_trail(result);
    } else {
        let n = result.paren_trails.len() - 1;
        let trail = result.paren_trails.get_mut(n).unwrap();
        trail.end_x = result.paren_trail.end_x.unwrap();
        if result.return_parens {
            if let Some(opener) = result.paren_trail.openers.last_mut() {
                opener.closer.as_mut().unwrap().trail = Some(trail.clone());
            }
        }
    }
}

fn finish_new_paren_trail<'a>(result: &mut State<'a>) {
    if result.is_in_stringish() {
        invalidate_paren_trail(result);
    } else if result.mode == Mode::Indent {
        clamp_paren_trail_to_cursor(result);
        pop_paren_trail(result);
    } else if result.mode == Mode::Paren {
        if let Some(paren) = peek(&result.paren_trail.openers, 0).map(Clone::clone) {
            set_max_indent(result, &paren);
        }
        if Some(result.line_no) != result.cursor_line {
            clean_paren_trail(result);
        }
        remember_paren_trail(result);
    }
}

// {{{1 Indentation functions

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
                Some(opener) if should_add_opener_indent(result, opener) => {
                    Some(opener.indent_delta)
                }
                _ => None,
            };

            if let Some(adjust) = to_add {
                add_indent(result, adjust);
            }
        }
        Mode::Paren => correct_indent(result),
    }

    Ok(())
}

fn check_leading_close_paren<'a>(result: &mut State<'a>) -> Result<()> {
    if result
        .error_pos_cache
        .contains_key(&ErrorName::LeadingCloseParen)
        && result.paren_trail.line_no == Some(result.line_no)
    {
        error(result, ErrorName::LeadingCloseParen)?;
    }

    Ok(())
}

fn on_leading_close_paren<'a>(result: &mut State<'a>) -> Result<()> {
    match result.mode {
        Mode::Indent => {
            if !result.force_balance {
                if result.smart {
                    error(result, ErrorName::Restart)?;
                }
                if !result
                    .error_pos_cache
                    .contains_key(&ErrorName::LeadingCloseParen)
                {
                    cache_error_pos(result, ErrorName::LeadingCloseParen);
                }
            }
            result.skip_char = true;
        }
        Mode::Paren => {
            if !is_valid_close_paren(&result.paren_stack, result.ch) {
                if result.smart {
                    result.skip_char = true;
                } else {
                    error(result, ErrorName::UnmatchedCloseParen)?;
                }
            } else if is_cursor_left_of(
                result.cursor_x,
                result.cursor_line,
                Some(result.x),
                result.line_no,
            ) {
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
                result.paren_stack.push(opener.clone());
            }
        }
    };

    let x = result.x;
    let i = get_parent_opener_index(result, x);
    let mut indent_to_add: Delta = 0;
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
    } else if result.ch == result.comment_char {
        // comments don't count as indentation points
        on_comment_line(result);
        result.tracking_indent = false;
    } else if result.ch != NEWLINE && result.ch != BLANK_SPACE && result.ch != TAB {
        on_indent(result)?;
    }

    Ok(())
}

fn make_tab_stop<'a>(opener: &Paren<'a>) -> TabStop<'a> {
    TabStop {
        ch: opener.ch,
        x: opener.x,
        line_no: opener.line_no,
        arg_x: opener.arg_x,
    }
}

fn get_tab_stop_line<'a>(result: &State<'a>) -> Option<LineNumber> {
    result.selection_start_line.or(result.cursor_line)
}

fn set_tab_stops<'a>(result: &mut State<'a>) {
    if get_tab_stop_line(result) != Some(result.line_no) {
        return;
    }

    result.tab_stops = result.paren_stack.iter().map(make_tab_stop).collect();

    if result.mode == Mode::Paren {
        let paren_trail_tabs: Vec<_> = result
            .paren_trail
            .openers
            .iter()
            .rev()
            .map(make_tab_stop)
            .collect();
        result.tab_stops.extend(paren_trail_tabs);
    }

    // remove argX if it falls to the right of the next stop
    for i in 1..result.tab_stops.len() {
        let x = result.tab_stops[i].x;
        if let Some(prev_arg_x) = result.tab_stops[i - 1].arg_x {
            if prev_arg_x >= x {
                result.tab_stops[i - 1].arg_x = None;
            }
        }
    }
}

// {{{1 High-level processing functions

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

    for (x, ch) in result.input_lines[line_no]
        .graphemes(true)
        .scan(0, |column, ch| {
            let start_column = *column;
            *column = *column + UnicodeWidthStr::width(ch);
            Some((start_column, ch))
        })
    {
        result.input_x = x;
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
    if result.quote_danger {
        error(result, ErrorName::QuoteDanger)?;
    }
    if result.is_in_stringish() {
        error(result, ErrorName::UnclosedQuote)?;
    }

    if result.paren_stack.len() != 0 {
        if result.mode == Mode::Paren {
            error(result, ErrorName::UnclosedParen)?;
        }
    }
    if result.mode == Mode::Indent {
        init_line(result);
        on_indent(result)?;
    }
    result.success = true;

    Ok(())
}

fn process_error<'a>(result: &mut State<'a>, e: Error) {
    result.success = false;
    result.error = Some(e);
}

fn process_text<'a>(text: &'a str, options: &Options, mode: Mode, smart: bool) -> State<'a> {
    let mut result = get_initial_result(text, &options, mode, smart);

    let mut process_result: Result<()> = Ok(());
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
        Err(Error {
            name: ErrorName::Restart,
            ..
        }) => process_text(text, &options, Mode::Paren, smart),
        Err(e) => {
            process_error(&mut result, e);
            result
        }
        _ => result,
    }
}

// {{{1 Public API

fn public_result<'a>(result: State<'a>) -> Answer<'a> {
    let line_ending = get_line_ending(result.orig_text);
    if result.success {
        Answer {
            text: Cow::from(result.lines.join(line_ending)),
            cursor_x: result.cursor_x,
            cursor_line: result.cursor_line,
            success: true,
            tab_stops: result.tab_stops,
            paren_trails: result.paren_trails,
            parens: result.parens,
            error: None,
        }
    } else {
        Answer {
            text: if result.partial_result {
                Cow::from(result.lines.join(line_ending))
            } else {
                Cow::from(result.orig_text)
            },
            cursor_x: if result.partial_result {
                result.cursor_x
            } else {
                result.orig_cursor_x
            },
            cursor_line: if result.partial_result {
                result.cursor_line
            } else {
                result.orig_cursor_line
            },
            paren_trails: result.paren_trails,
            success: false,
            tab_stops: result.tab_stops,
            error: result.error,
            parens: result.parens,
        }
    }
}

pub fn indent_mode<'a>(text: &'a str, options: &Options) -> Answer<'a> {
    public_result(process_text(text, options, Mode::Indent, false))
}

pub fn paren_mode<'a>(text: &'a str, options: &Options) -> Answer<'a> {
    public_result(process_text(text, options, Mode::Paren, false))
}

pub fn smart_mode<'a>(text: &'a str, options: &Options) -> Answer<'a> {
    let smart = options.selection_start_line == None;
    public_result(process_text(text, options, Mode::Indent, smart))
}

pub fn process(request: &Request) -> Answer {
    let mut options = request.options.clone();

    if let Some(ref prev_text) = request.options.prev_text {
        options.changes = changes::compute_text_changes(prev_text, &request.text);
    }

    if request.mode == "paren" {
        Answer::from(paren_mode(&request.text, &options))
    } else if request.mode == "indent" {
        Answer::from(indent_mode(&request.text, &options))
    } else if request.mode == "smart" {
        Answer::from(smart_mode(&request.text, &options))
    } else {
        Answer::from(Error {
            message: String::from("Bad value specified for `mode`"),
            ..Error::default()
        })
    }
}

// This is like the process function above, but uses a reference counted version of Request
#[allow(dead_code)]
pub fn rc_process<'a>(request: &'a SharedRequest) -> Answer<'a> {
  let mut options = request.options.clone();

  if let Some(ref prev_text) = request.options.prev_text {
    options.changes = changes::compute_text_changes(prev_text, &request.text);
  }

  if request.mode == "paren" {
    Answer::from(paren_mode(&request.text, &options))
  } else if request.mode == "indent" {
    Answer::from(indent_mode(&request.text, &options))
  } else if request.mode == "smart" {
    Answer::from(smart_mode(&request.text, &options))
  } else {
    Answer::from(Error {
      message: String::from("Bad value specified for `mode`"),
      ..Error::default()
    })
  }
}
