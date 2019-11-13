use parinfer::chomp_cr;
use types::*;
use std::env;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Coord {
    pub line: LineNumber,
    pub column: Column
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Selection {
    pub anchor: Coord,
    pub cursor: Coord
}

impl Selection {
    fn new(
        anchor_line: LineNumber, anchor_column: Column, cursor_line: LineNumber,
        cursor_column: Column) -> Selection
    {
        Selection {
            anchor: Coord {
                line: anchor_line,
                column: anchor_column
            },
            cursor: Coord {
                line: cursor_line,
                column: cursor_column
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Insertion {
    pub cursor: Coord,
    pub text: String
}

impl Insertion {
    fn new(cursor_line: LineNumber, cursor_column: Column, text: &str) -> Insertion {
        Insertion {
            cursor: Coord {
                line: cursor_line,
                column: cursor_column
            },
            text: text.to_string()
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fixes {
    pub deletions: Vec<Selection>,
    pub insertions: Vec<Insertion>
}

pub fn fixes<'a>(from: &'a str, to: &'a str) -> Fixes {
    let mut result = Fixes {
        insertions: vec![],
        deletions: vec![]
    };

    let mut line: LineNumber = 1;
    for (a_line, b_line) in from.split('\n').map(chomp_cr).zip(to.split('\n').map(chomp_cr)) {
        if a_line != b_line {
            result.deletions.push(Selection::new(line, 1, line, a_line.chars().count()));
            if b_line != "" {
                result.insertions.push(Insertion::new(line, 1, b_line));
            }
        }
        line += 1;
    }

    result
}

fn escape(s: &str) -> String {
    s.replace("'", "''")
}

fn delete_script(fixes: &Fixes) -> String {
    if fixes.deletions.is_empty() {
        String::new()
    } else {
        format!(
            "select {} {}\nexec '\\<a-d>'\n",
            env::var("kak_opt_parinfer_select_switches").unwrap(),
            fixes
                .deletions
                .iter()
                .map(|d| {
                    format!(
                        "{}.{},{}.{}",
                        d.anchor.line,
                        d.anchor.column,
                        d.cursor.line,
                        d.cursor.column
                    )
                })
                .fold(String::new(), |acc, s| acc + " " + &s)
        )
    }
}

fn insert_script(fixes: &Fixes) -> String {
    if fixes.insertions.is_empty() {
        String::new()
    } else {
        format!(
            "select {} {}
             set-register '\"' {}
             exec '\\P'",
            env::var("kak_opt_parinfer_select_switches").unwrap(),
            fixes
                .insertions
                .iter()
                .map(|i| {
                    format!(
                        "{}.{},{}.{}",
                        i.cursor.line,
                        i.cursor.column,
                        i.cursor.line,
                        i.cursor.column
                    )
                })
                .fold(String::new(), |acc, s| acc + " " + &s),
            fixes
                .insertions
                .iter()
                .map(|i| {
                    format!("'{}'", escape(&i.text))
                })
                .fold(String::new(), |acc, s| acc + " " + &s)
        )
    }
}

fn cursor_script(request: &Request, answer: &Answer) -> String {
    match (request.options.cursor_line, request.options.cursor_x, answer.cursor_line, answer.cursor_x) {
        (Some(r_line), Some(r_x), Some(a_line), Some(a_x)) if r_line == a_line && r_x == a_x => String::new(),
        (_, _, Some(line), Some(x)) => format!("set buffer parinfer_cursor_char_column {}
                                                set buffer parinfer_cursor_line {}",
                                               x + 1, line + 1),
        _ => String::new()
    }
}

pub fn kakoune_output(request: &Request, answer: Answer) -> (String, i32) {
    if answer.success {
        let fixes = fixes(&request.text, &answer.text);
        let script = format!("{}\n{}\n{}", delete_script(&fixes), insert_script(&fixes),
                             cursor_script(&request, &answer));

        ( script, 0 )
    } else {
        let error_msg = match answer.error {
            None => String::from("unknown error."),
            Some(e) => e.message
        };

        ( format!("fail '{}'\n", escape(&error_msg)), 0 )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn fixes_works() {
        assert_eq!(
            fixes("abc", "abc"),
            Fixes {
                deletions: vec![],
                insertions: vec![]
            },
            "it can handle no changes"
        );
        assert_eq!(
            fixes("abcd", "axcy"),
            Fixes {
                deletions: vec![
                    Selection::new(1,1,1,4),
                ],
                insertions: vec![
                    Insertion::new(1,1,"axcy"),
                ]
            },
            "it can produce a replacement for a single changed letter"
        );
        assert_eq!(
            fixes("hello, worxxyz", ""),
            Fixes {
                deletions: vec![
                    Selection::new(1,1,1,14)
                ],
                insertions: vec![]
            },
            "it can produce a longer deletion"
        );
    }
}
