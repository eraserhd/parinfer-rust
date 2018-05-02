use super::compute_text_change;
use parinfer;
use std;
use serde_json;
use json::*;

fn internal_run(input: String) -> Result<String, Error> {
    let request: Request = serde_json::from_str(&input)?;
    let mut options = request.options.to_parinfer();

    if let Some(ref prev_text) = request.options.prev_text {
        options.changes.clear();
        if let Some(change) = compute_text_change(prev_text, &request.text) {
            options.changes.push(change);
        }
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

    let response = serde_json::to_string(&Answer::from(answer))?;

    Ok(response)
}

pub fn run_parinfer(input: String) -> String {
    let answer_string: String = match internal_run(input) {
        Ok(result) => result,
        Err(e) => serde_json::to_string(&Answer {
            text: std::borrow::Cow::from(""),
            success: false,
            error: Some(e),
            cursor_x: None,
            cursor_line: None,
            tab_stops: vec![],
            paren_trails: vec![],
            parens: vec![]
        }).unwrap()
    };
    answer_string
}

#[cfg(test)]
mod tests {
    use super::run_parinfer;
    use serde_json;
    use serde_json::{Value};

    #[test]
    fn it_works() {
        let out = run_parinfer(String::from(r#"{
            "mode": "indent",
            "text": "(def x",
            "options": {
                "cursorX": 3,
                "cursorLine": 0
            }
        }"#));
        let answer : Value = serde_json::from_str(&out).unwrap();
        assert_eq!(
            Value::Bool(true),
            answer["success"],
            "successfully runs parinfer"
        );
        assert_eq!(
            Value::String(String::from("(def x)")),
            answer["text"],
            "returns correct text"
        );
    }
}
