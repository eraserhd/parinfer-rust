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
        if result.is_empty() {
            result.push(Change {
               leader: String::new(),
               added: String::new(),
               removed: String::new()
            });
        }
        match change {
            Difference::Same(s) => {
                result.last_mut().unwrap().leader += &s;
            },
            Difference::Add(s) => {
                result.last_mut().unwrap().added += &s;
            }
            _ => ()
        }
    }
    result
}

#[cfg(test)]
#[test]
pub fn group_changeset_works() {
    assert_eq!(group_changeset(vec![]), vec![]);
    assert_eq!(
        group_changeset(vec![Difference::Same("hello".to_string())]),
        vec![Change {
            leader: String::from("hello"),
            added: String::from(""),
            removed: String::from("")
        }]
    );
    assert_eq!(
        group_changeset(vec![
            Difference::Same("hello".to_string()),
            Difference::Same(", world".to_string())
        ]),
        vec![Change {
            leader: String::from("hello, world"),
            added: String::from(""),
            removed: String::from("")
        }]
    );
    assert_eq!(
        group_changeset(vec![
            Difference::Same("hello".to_string()),
            Difference::Add("there".to_string()),
            Difference::Add("!".to_string())
        ]),
        vec![Change {
            leader: "hello".to_string(),
            added: "there!".to_string(),
            removed: "".to_string()
        }]
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
