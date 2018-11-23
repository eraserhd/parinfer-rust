use parinfer::chomp_cr;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Coord {
    pub line: u64,
    pub column: u64
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Selection {
    pub anchor: Coord,
    pub cursor: Coord
}

impl Selection {
    fn new(
        anchor_line: u64, anchor_column: u64, cursor_line: u64,
        cursor_column: u64) -> Selection
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
    fn new(cursor_line: u64, cursor_column: u64, text: &str) -> Insertion {
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

    let mut line: u64 = 1; // LineNumber type
    for (a_line, b_line) in from.split('\n').map(chomp_cr).zip(to.split('\n').map(chomp_cr)) {
        if a_line != b_line {
            result.deletions.push(Selection {
              anchor: Coord {
                  line,
                  column: 1
              },
              cursor: Coord {
                  line,
                  column: a_line.chars().count() as u64 // type
              }
            });
            if b_line != "" {
                result.insertions.push(Insertion {
                   cursor: Coord {
                       line,
                       column: 1
                   },
                   text: String::from(b_line)
                });
            }
        }
        line += 1;
    }

    result
}

#[cfg(test)]
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
