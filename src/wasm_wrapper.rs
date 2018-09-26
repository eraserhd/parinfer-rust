use types::*;
use serde_json;
use std::panic;
use super::common_wrapper;

pub fn run_parinfer(input: String) -> String {
    match panic::catch_unwind(|| common_wrapper::internal_run(&input)) {
        Ok(Ok(result)) => result,
        Ok(Err(e)) => serde_json::to_string(&Answer::from(e)).unwrap(),
        Err(_) => common_wrapper::panic_result()
    }
}

#[cfg(test)]
mod tests {
    use super::run_parinfer;
    use serde_json;
    use serde_json::Value;

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
