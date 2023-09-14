use super::*;
use std::borrow::Cow;
use types::*;

pub fn internal_run(json_str: &str) -> Result<String, Error> {
    let request: Request = serde_json::from_str(json_str)?;
    let answer = parinfer::process(&request);
    Ok(serde_json::to_string(&Answer::from(answer))?)
}

pub fn panic_result() -> String {
    let answer = Answer {
        text: Cow::from(""),
        success: false,
        error: Some(Error {
            name: ErrorName::Panic,
            message: String::from("plugin panicked!"),
            x: 0,
            line_no: 0,
            input_x: 0,
            input_line_no: 0,
        }),
        cursor_x: None,
        cursor_line: None,
        tab_stops: vec![],
        paren_trails: vec![],
        parens: vec![]
    };

    serde_json::to_string(&answer).unwrap()
}
