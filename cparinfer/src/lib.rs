extern crate parinfer;

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

extern crate libc;
use libc::c_char;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Change {
    x: parinfer::Column,
    line_no: parinfer::LineNumber,
    old_text: String,
    new_text: String,
}

impl Change {
    fn to_parinfer(&self) -> parinfer::Change {
        parinfer::Change {
            x: self.x,
            line_no: self.line_no,
            old_text: &self.old_text,
            new_text: &self.new_text
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Options {
    cursor_x: Option<parinfer::Column>,
    cursor_line: Option<parinfer::LineNumber>,
    prev_cursor_x: Option<parinfer::Column>,
    prev_cursor_line: Option<parinfer::LineNumber>,
    selection_start_line: Option<parinfer::LineNumber>,
    changes: Vec<Change>,
    #[serde(default = "Options::default_false")]
    partial_result: bool,
    #[serde(default = "Options::default_false")]
    force_balance: bool,
    #[serde(default = "Options::default_false")]
    return_parens: bool
}

impl Options {
    fn default_false() -> bool {
        false
    }

    fn to_parinfer(&self) -> parinfer::Options {
        parinfer::Options {
            cursor_x: self.cursor_x,
            cursor_line: self.cursor_line,
            prev_cursor_x: self.prev_cursor_x,
            prev_cursor_line: self.prev_cursor_line,
            selection_start_line: self.selection_start_line,
            changes: self.changes.iter().map(Change::to_parinfer).collect(),
            partial_result: self.partial_result,
            force_balance: self.force_balance,
            return_parens: self.return_parens
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Request {
    mode: String,
    text: String,
    options: Options
}

#[no_mangle]
pub extern "C" fn run_parinfer(json: *const c_char) -> *const c_char {
    unimplemented!();
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
