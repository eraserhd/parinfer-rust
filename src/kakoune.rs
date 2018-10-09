use text_diff::*;
#[derive(Debug, Eq, PartialEq)]
pub struct Coord {
    line: u64,
    column: u64
}

#[derive(Debug, Eq, PartialEq)]
pub struct Selection {
    anchor: Coord,
    cursor: Coord
}

#[derive(Debug, Eq, PartialEq)]
pub struct Replacement {
    selection: Selection,
    text: String
}

/// A ChangeGroup is a (possibly empty) bit of unchanged leading text followed
/// by added and removed text.
///
/// The order of the added and removed text doesn't matter to us since we want
/// to make one big replace or delete from it.
#[derive(Debug, Eq, PartialEq)]
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


pub fn replacements<'a>(from: &'a str, to: &'a str) -> Vec<Replacement> {
    vec![]
}

#[cfg(test)]
#[test]
pub fn replacements_works() {
    assert_eq!(replacements("abc", "abc"), vec![]);
}
