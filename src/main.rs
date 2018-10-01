extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

extern crate unicode_segmentation;
extern crate unicode_width;


mod parinfer;
mod types;
mod changes;
mod cli_options;

use std::env;
use std::io;
use std::io::Write;
use cli_options::OutputType;

extern crate getopts;

fn parse_args() -> cli_options::Options {
    let args: Vec<String> = env::args().collect();
    cli_options::Options::parse(&args[1..])
        .expect("failed to parse options")
}

pub fn main() {
    let opts = parse_args();
    if opts.want_help() {
        print!("{}", cli_options::usage());
    } else {
        let request = opts.request().expect("unable to parse options");
        let answer = parinfer::process(&request);
        let output = match opts.output_type() {
            OutputType::Json => serde_json::to_string(&answer).expect("unable to produce JSON"),
            OutputType::Text => String::from(answer.text)
        };
        io::stdout().write(output.as_bytes()).expect("unable to write output");
    }
}
