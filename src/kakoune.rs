use text_diff::*;

/// A ChangeGroup is a (possibly empty) bit of unchanged leading text followed
/// by added and removed text.
///
/// The order of the added and removed text doesn't matter to us since we want
/// to make one big replace or delete from it.
#[derive(Clone, Debug, Eq, PartialEq)]
struct ChangeGroup {
    unchanged_leader: String,
    added_text: String,
    removed_text: String
}

impl ChangeGroup {
    fn has_changes(&self) -> bool {
        !self.added_text.is_empty() || !self.removed_text.is_empty()
    }

    fn new() -> ChangeGroup {
        ChangeGroup {
            unchanged_leader: String::new(),
            added_text: String::new(),
            removed_text: String::new()
        }
    }
}

fn group_changeset(changeset: Vec<Difference>) -> Vec<ChangeGroup> {
    let mut result: Vec<ChangeGroup> = vec![ChangeGroup::new()];
    for change in changeset {
        match change {
            Difference::Same(s) => {
                if result.last().unwrap().has_changes() {
                    result.push(ChangeGroup::new());
                }
                result.last_mut().unwrap().unchanged_leader += &s;
            },
            Difference::Add(s) => {
                result.last_mut().unwrap().added_text += &s;
            },
            Difference::Rem(s) => {
                result.last_mut().unwrap().removed_text += &s;
            }
        }
    }
    if !result.last().unwrap().has_changes() {
        result.pop();
    }
    result
}

#[cfg(test)]
#[test]
pub fn group_changeset_works() {
    assert_eq!(group_changeset(vec![]), vec![]);
    assert_eq!(
        group_changeset(vec![
            Difference::Same("hello".to_string()),
            Difference::Same(", world".to_string()),
            Difference::Add("foo".to_string())
        ]),
        vec![ChangeGroup {
            unchanged_leader: String::from("hello, world"),
            added_text: String::from("foo"),
            removed_text: String::from("")
        }],
        "it collects and combines unchanged_leader text"
    );
    assert_eq!(
        group_changeset(vec![
            Difference::Same("hello".to_string()),
            Difference::Add("there".to_string()),
            Difference::Add("!".to_string())
        ]),
        vec![ChangeGroup {
            unchanged_leader: "hello".to_string(),
            added_text: "there!".to_string(),
            removed_text: "".to_string()
        }],
        "it collects and combines added_text text"
    );
    assert_eq!(
        group_changeset(vec![
            Difference::Rem("hello".to_string()),
            Difference::Rem("there".to_string()),
            Difference::Add("!".to_string())
        ]),
        vec![ChangeGroup {
            unchanged_leader: "".to_string(),
            added_text: "!".to_string(),
            removed_text: "hellothere".to_string()
        }],
        "it collects and combines removed_text text"
    );
    assert_eq!(
        group_changeset(vec![
            Difference::Rem("hello".to_string()),
            Difference::Same("there".to_string()),
            Difference::Add("!".to_string())
        ]),
        vec![ChangeGroup {
            unchanged_leader: "".to_string(),
            added_text: "".to_string(),
            removed_text: "hello".to_string()
        }, ChangeGroup {
            unchanged_leader: "there".to_string(),
            added_text: "!".to_string(),
            removed_text: "".to_string()
        }],
        "it starts a new change when seeing a 'Same' node"
    );
    assert_eq!(
        group_changeset(vec![
            Difference::Rem("hello".to_string()),
            Difference::Same("there".to_string()),
            Difference::Same("!".to_string())
        ]),
        vec![ChangeGroup {
            unchanged_leader: "".to_string(),
            added_text: "".to_string(),
            removed_text: "hello".to_string()
        }],
        "it doesn't return a trailing node without changes"
    );
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Coord {
    line: u64,
    column: u64
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Selection {
    anchor: Coord,
    cursor: Coord
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
    cursor: Coord,
    text: String
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
    deletions: Vec<Selection>,
    insertions: Vec<Insertion>
}

fn advance(pos: Coord, s: &str) -> (Coord, Coord) {
    let mut pos = pos;
    let mut previous = pos.clone();
    for ch in s.chars() {
        previous = pos.clone();
        if ch == '\n' {
            pos.line += 1;
            pos.column = 1;
        } else {
            pos.column += 1;
        }
    }
    (previous, pos)
}

pub fn fixes<'a>(from: &'a str, to: &'a str) -> Fixes {
    let (_, changeset) = diff(from, to, "");
    let mut result = Fixes {
        insertions: vec![],
        deletions: vec![]
    };
    let mut pos = Coord {
        line: 1,
        column: 1
    };
    let change_groups = group_changeset(changeset);
    for change_group in change_groups.clone() {
        pos = advance(pos, &change_group.unchanged_leader).1;
        if !change_group.removed_text.is_empty() {
            let anchor = pos.clone();
            let advanced = advance(pos, &change_group.removed_text);
            pos = advanced.1;
            let cursor = advanced.0;

            result.deletions.push(Selection {
                anchor,
                cursor
            });
        }
    }
    pos = Coord {
        line: 1,
        column: 1
    };
    for change_group in change_groups {
        pos = advance(pos, &change_group.unchanged_leader).1;
        if !change_group.added_text.is_empty() {
            result.insertions.push(Insertion {
                cursor: pos.clone(),
                text: change_group.added_text
            });
        }
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
                Selection::new(1,2,1,2),
                Selection::new(1,4,1,4)
            ],
            insertions: vec![
                Insertion::new(1,2,"x"),
                Insertion::new(1,3,"y")
            ]
        },
        "it can produce a replacement for a single changed letter"
    );
    assert_eq!(
        fixes("hello, worxxyz", "herxx"),
        Fixes {
            deletions: vec![
                Selection::new(1,3,1,9),
                Selection::new(1,13,1,14)
            ],
            insertions: vec![]
        },
        "it can produce a longer deletion"
    );
}
