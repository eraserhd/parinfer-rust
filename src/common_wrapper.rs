use super::*;
use std::borrow::Cow;
use types::*;
use changes;

pub fn process(request: &Request) -> Result<Answer, Error> {
    let mut options = request.options.to_parinfer();

    if let Some(ref prev_text) = request.options.prev_text {
        options.changes = changes::compute_text_changes(prev_text, &request.text);
    }

    if request.mode == "paren" {
        Ok(Answer::from(parinfer::paren_mode(&request.text, &options)))
    } else if request.mode == "indent" {
        Ok(Answer::from(parinfer::indent_mode(&request.text, &options)))
    } else if request.mode == "smart" {
        Ok(Answer::from(parinfer::smart_mode(&request.text, &options)))
    } else {
        Err(Error {
            message: String::from("Bad value specified for `mode`"),
            ..Error::default()
        })
    }
}

pub fn internal_run(json_str: &str) -> Result<String, Error> {
    let request: Request = serde_json::from_str(json_str)?;
    let answer = process(&request)?;
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
