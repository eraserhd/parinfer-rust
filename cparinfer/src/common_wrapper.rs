use super::*;
use json::*;
use changes;

pub fn internal_run(json_str: &str) -> Result<String, Error> {
    let request: Request = serde_json::from_str(json_str)?;
    let mut options = request.options.to_parinfer();

    if let Some(ref prev_text) = request.options.prev_text {
        options.changes.clear();
        if let Some(change) = changes::compute_text_change(prev_text, &request.text) {
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

    Ok(serde_json::to_string(&Answer::from(answer))?)
}

