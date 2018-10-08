extern crate getopts;

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

extern crate text_diff;

extern crate unicode_segmentation;
extern crate unicode_width;


mod parinfer;
mod types;
mod changes;
mod cli_options;

use std::env;
use std::io;
use std::io::Write;
use types::*;
use cli_options::OutputType;

fn parse_args() -> cli_options::Options {
    let args: Vec<String> = env::args().collect();
    cli_options::Options::parse(&args[1..])
        .expect("failed to parse options")
}

fn json_output(_request: &Request, answer: Answer) -> (String, i32) {
    let text = serde_json::to_string(&answer).expect("unable to produce JSON");
    let error_code = if answer.success { 0 } else { 1 };
    ( text, error_code )
}

fn kakoune_escape(s: &str) -> String {
    s.replace("'", "''")
}

fn kakoune_output(_request: &Request, answer: Answer) -> (String, i32) {
    if answer.success {
        ( format!("exec '%' ; set-register '\"' '{}' ; exec -draft '\\R'",
                  kakoune_escape(&answer.text)),
          0 )
    } else {
        let error_msg = match answer.error {
            None => String::from("unknown error."),
            Some(e) => e.message
        };

        ( format!("fail '{}'\n", kakoune_escape(&error_msg)), 0 )
    }
}

fn text_output(_request: &Request, answer: Answer) -> (String, i32) {
    if answer.success {
        ( answer.text.into_owned(), 0 )
    } else {
        match answer.error {
            None => ( String::from("parinfer-rust: unknown error.\n"), 1 ),
            Some(e) => ( format!("parinfer-rust: {}\n", e.message), 1 )
        }
    }
}

pub fn main() {
    let opts = parse_args();
    if opts.want_help() {
        print!("{}", cli_options::usage());
    } else {
        let request = opts.request().expect("unable to parse options");
        let answer = parinfer::process(&request);
        let (output, error_code) = match opts.output_type() {
            OutputType::Json => json_output(&request, answer),
            OutputType::Kakoune => kakoune_output(&request, answer),
            OutputType::Text => text_output(&request, answer)
        };
        io::stdout().write(output.as_bytes()).expect("unable to write output");
        std::process::exit(error_code);
    }
}
