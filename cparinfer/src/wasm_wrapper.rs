use std;
use serde_json;
use json::*;
use super::common_wrapper;

pub fn run_parinfer(input: String) -> String {
    let answer_string: String = match common_wrapper::internal_run(&input) {
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
