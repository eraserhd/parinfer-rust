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
    for change in changeset {
        match change {
            Difference::Same(s) => {
                result.push(Change {
                   leader: s,
                   added: String::new(),
                   removed: String::new()
                });
            },
            _ => ()
        }
    }
    result
}

#[cfg(test)]
#[test]
pub fn group_changeset_works() {
    assert_eq!(group_changeset(vec![]), vec![]);
    assert_eq!(group_changeset(vec![Difference::Same("hello".to_string())]),
               vec![Change {
                   leader: String::from("hello"),
                   added: String::from(""),
                   removed: String::from("")
               }]);
}


pub fn replacements<'a>(from: &'a str, to: &'a str) -> Vec<Replacement> {
    vec![]
}

#[cfg(test)]
#[test]
pub fn replacements_works() {
    assert_eq!(replacements("abc", "abc"), vec![]);
}
