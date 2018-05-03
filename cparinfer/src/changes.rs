use super::parinfer;

pub fn compute_text_change<'a>(prev_text: &'a str, text: &'a str) -> Option<parinfer::Change<'a>> {
    let mut x: parinfer::Column = 0;
    let mut line_no: parinfer::LineNumber = 0;
    let mut start_text: usize = 0;
    let mut start_prev: usize = 0;
    let mut end_text: usize = text.len();
    let mut end_prev: usize = prev_text.len();
    let mut different: bool = false;

    for ((i, pc), (j, c)) in prev_text.char_indices().zip(text.char_indices()) {
        if pc != c {
            start_prev = i;
            start_text = j;
            different = true;
            break;
        }
        if pc == '\n' {
            x = 0;
            line_no += 1;
        } else {
            x += 1;
        }
    }

    for ((i, pc), (j, c)) in prev_text.char_indices().rev().zip(text.char_indices().rev()) {
        if pc != c || i < start_prev || j < start_text {
            end_prev = i + pc.len_utf8();
            end_text = j + c.len_utf8();
            break;
        }
    }

    if different {
        Some(parinfer::Change {
            x,
            line_no,
            old_text: &prev_text[start_prev..end_prev],
            new_text: &text[start_text..end_text]
        })
    } else {
        None
    }
}

#[cfg(test)]
#[test]
fn compute_text_change_works() {
    assert_eq!(None, compute_text_change("hello", "hello"));
    assert_eq!(Some(parinfer::Change {
        x: 2,
        line_no: 0,
        old_text: "l",
        new_text: "x"
    }), compute_text_change("hello", "hexlo"));
    assert_eq!(Some(parinfer::Change {
        x: 0,
        line_no: 1,
        old_text: "l",
        new_text: "x"
    }), compute_text_change("he\nllo", "he\nxlo"));
    assert_eq!(Some(parinfer::Change {
        x: 4,
        line_no: 0,
        old_text: "",
        new_text: "l"
    }), compute_text_change("hello", "helllo"));
    assert_eq!(Some(parinfer::Change {
        x: 4,
        line_no: 0,
        old_text: "l",
        new_text: ""
    }), compute_text_change("helllo", "hello"));
}

