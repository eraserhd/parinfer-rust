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

#[derive(Debug, Eq, PartialEq)]
struct Change {
    leader: String,
    added: String,
    removed: String
}

fn group_changeset(changeset: Vec<Difference>) -> Vec<Change> {
    let mut result: Vec<Change> = vec![];
    result
}

#[cfg(test)]
#[test]
pub fn group_changeset_works() {
    assert_eq!(group_changeset(vec![]), vec![]);
}


pub fn replacements<'a>(from: &'a str, to: &'a str) -> Vec<Replacement> {
    vec![]
}

#[cfg(test)]
#[test]
pub fn replacements_works() {
    assert_eq!(replacements("abc", "abc"), vec![]);
}
