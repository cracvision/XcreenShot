//! The sequence of step markers of a document (EDT-035): which value the next click
//! places, Restart sequence and Renumber.
//!
//! The next value is the last one placed plus one. A document opened anew has none:
//! its next value is the largest of that sequence in it plus one.

use std::collections::BTreeMap;

use serde_json::{json, Value};
use xs_core::document::{Document, Object};
use xs_image::steps::Sequence;

/// The sequence of a step object, from its style.
pub fn sequence_of(o: &Object) -> Sequence {
    o.props
        .get("style")
        .and_then(|s| s.get("sequence"))
        .and_then(Value::as_str)
        .and_then(Sequence::from_key)
        .unwrap_or(Sequence::Number)
}

fn value_of(o: &Object) -> &str {
    o.props.get("value").and_then(Value::as_str).unwrap_or_default()
}

fn steps(doc: &Document, seq: Sequence) -> impl Iterator<Item = &Object> {
    doc.objects
        .iter()
        .filter(move |o| o.kind == "step" && sequence_of(o) == seq)
}

/// The next value of `seq` in `doc`, as a number of the sequence. `next` is what the
/// last placed or edited value left, per sequence key.
pub fn next(doc: &Document, seq: Sequence, next: &BTreeMap<String, u64>) -> u64 {
    next.get(seq.key()).copied().unwrap_or_else(|| {
        steps(doc, seq)
            .filter_map(|o| seq.index(value_of(o)))
            .max()
            .unwrap_or(0)
            + 1
    })
}

/// The next value of each sequence, as text: what Properties shows.
pub fn next_values(doc: &Document, next_map: &BTreeMap<String, u64>) -> Value {
    let m: serde_json::Map<String, Value> = Sequence::ALL
        .iter()
        .map(|s| (s.key().to_string(), json!(s.value(next(doc, *s, next_map)))))
        .collect();
    Value::Object(m)
}

/// The object number of an id (`o12` → 12): the order objects were made in.
fn made(o: &Object) -> u64 {
    o.id.strip_prefix('o')
        .and_then(|n| n.parse().ok())
        .unwrap_or(u64::MAX)
}

/// Numbers again the steps of `seq` by the order they were made, without gaps. Returns
/// how many.
pub fn renumber(doc: &mut Document, seq: Sequence) -> u64 {
    let mut ids: Vec<(u64, String)> = steps(doc, seq).map(|o| (made(o), o.id.clone())).collect();
    ids.sort();
    for (n, (_, id)) in ids.iter().enumerate() {
        if let Some(o) = doc.object_mut(id) {
            o.props.insert("value".into(), json!(seq.value(n as u64 + 1)));
        }
    }
    ids.len() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn step(id: &str, value: &str) -> Object {
        Object {
            id: id.into(),
            kind: "step".into(),
            props: json!({"value": value, "style": {"sequence": "number"}})
                .as_object()
                .cloned()
                .unwrap(),
        }
    }

    #[test]
    fn the_next_value_follows_the_largest_and_what_was_placed() {
        let mut d = Document::new(10, 10, 96.0);
        d.objects = vec![step("o1", "1"), step("o2", "B2"), step("o3", "5")];
        let mut m = BTreeMap::new();
        assert_eq!(next(&d, Sequence::Number, &m), 6);
        assert_eq!(next(&d, Sequence::Upper, &m), 1, "no letters yet");
        m.insert("number".into(), 2);
        assert_eq!(next(&d, Sequence::Number, &m), 2);
    }

    #[test]
    fn renumbering_closes_the_gaps_in_the_order_they_were_made() {
        let mut d = Document::new(10, 10, 96.0);
        d.objects = vec![
            step("o10", "5"),
            step("o2", "1"),
            step("o3", "2"),
            step("o9", "4"),
        ];
        assert_eq!(renumber(&mut d, Sequence::Number), 4);
        let values: Vec<&str> = d.objects.iter().map(value_of).collect();
        assert_eq!(values, ["4", "1", "2", "3"]);
    }
}
