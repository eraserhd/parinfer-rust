extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

extern crate unicode_segmentation;
extern crate unicode_width;


mod parinfer;
mod json;
mod changes;
mod common_wrapper;

use std::env;
use std::io;
use std::io::{Read,Write};
use std::panic;

extern crate getopts;

fn options() -> getopts::Options {
    let mut options = getopts::Options::new();
    options.optflag("h", "help", "show this help message");
    options.optflag("j", "json", "read JSON input and write JSON response");
    options
}

fn parse_args() -> getopts::Matches {
    let args: Vec<String> = env::args().collect();
    match options().parse(&args[1..]) {
        Ok(matches) => matches,
        Err(f) => { panic!(f.to_string()); }
    }
}

pub fn main() -> io::Result<()> {
    let matches = parse_args();
    if matches.opt_present("h") {
        print!("{}", options().usage("Usage: parinfer-rust [options]"));
    } else if matches.opt_present("j") {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        let output = match panic::catch_unwind(|| common_wrapper::internal_run(&input)) {
            Ok(Ok(result)) => result,
            Ok(Err(e)) => serde_json::to_string(&json::Answer::from(e)).unwrap(),
            Err(_) => common_wrapper::panic_result()
        };
        io::stdout().write(output.as_bytes())?;
    } else {
        let mut text = String::new();
        io::stdin().read_to_string(&mut text)?;
        let options = parinfer::Options {
            changes: vec![],
            cursor_x: None,
            cursor_line: None,
            prev_cursor_x: None,
            prev_cursor_line: None,
            force_balance: false,
            return_parens: false,
            partial_result: false,
            selection_start_line: None
        };
        let answer = parinfer::indent_mode(&text, &options);
        io::stdout().write(answer.text.as_bytes())?;
    }
    Ok(())
}
