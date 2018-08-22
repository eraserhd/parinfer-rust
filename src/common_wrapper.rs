use super::*;
use std::borrow::Cow;
use json::*;
use changes;

pub fn internal_run(json_str: &str) -> Result<String, Error> {
    let request: Request = serde_json::from_str(json_str)?;
    let mut options = request.options.to_parinfer();

    if let Some(ref prev_text) = request.options.prev_text {
        options.changes = changes::compute_text_changes(prev_text, &request.text);
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

    Ok(serde_json::to_string(&Answer::from(answer))?)
}

pub fn panic_result() -> String {
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

    serde_json::to_string(&answer).unwrap()
}
