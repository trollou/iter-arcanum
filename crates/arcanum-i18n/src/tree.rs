//! Lecture du fichier JSON en arbre, puis aplatissement en clés pointées.
//! `serde_json::Value` garderait silencieusement la dernière clé en double :
//! on lit donc les objets en listes de couples pour pouvoir la refuser.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::de::{self, Deserialize, Deserializer, IgnoredAny, MapAccess, SeqAccess, Visitor};

use crate::{LoadError, is_valid_name};

enum Node {
    Text(String),
    Object(Vec<(String, Node)>),
    Other,
}

impl<'de> Deserialize<'de> for Node {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(NodeVisitor)
    }
}

struct NodeVisitor;

impl<'de> Visitor<'de> for NodeVisitor {
    type Value = Node;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("une valeur JSON")
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Node, E> {
        Ok(Node::Text(value.to_owned()))
    }

    fn visit_string<E: de::Error>(self, value: String) -> Result<Node, E> {
        Ok(Node::Text(value))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Node, A::Error> {
        let mut entries = Vec::new();
        while let Some((name, node)) = map.next_entry::<String, Node>()? {
            entries.push((name, node));
        }
        Ok(Node::Object(entries))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Node, A::Error> {
        while seq.next_element::<IgnoredAny>()?.is_some() {}
        Ok(Node::Other)
    }

    fn visit_bool<E: de::Error>(self, _: bool) -> Result<Node, E> {
        Ok(Node::Other)
    }

    fn visit_i64<E: de::Error>(self, _: i64) -> Result<Node, E> {
        Ok(Node::Other)
    }

    fn visit_u64<E: de::Error>(self, _: u64) -> Result<Node, E> {
        Ok(Node::Other)
    }

    fn visit_f64<E: de::Error>(self, _: f64) -> Result<Node, E> {
        Ok(Node::Other)
    }

    fn visit_unit<E: de::Error>(self) -> Result<Node, E> {
        Ok(Node::Other)
    }
}

/// Clés pointées → texte brut des libellés.
pub(crate) fn parse(json: &str) -> Result<BTreeMap<String, String>, LoadError> {
    let root: Node =
        serde_json::from_str(json).map_err(|error| LoadError::InvalidJson(error.to_string()))?;
    let Node::Object(entries) = root else {
        return Err(LoadError::RootNotObject);
    };
    let mut labels = BTreeMap::new();
    flatten("", entries, &mut labels)?;
    Ok(labels)
}

fn flatten(
    prefix: &str,
    entries: Vec<(String, Node)>,
    labels: &mut BTreeMap<String, String>,
) -> Result<(), LoadError> {
    let mut seen = BTreeSet::new();
    for (name, node) in entries {
        let key = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}.{name}")
        };
        if !is_valid_name(&name) {
            return Err(LoadError::InvalidKeyName { key });
        }
        if !seen.insert(name) {
            return Err(LoadError::DuplicateKey { key });
        }
        match node {
            Node::Text(text) => {
                labels.insert(key, text);
            }
            Node::Object(children) => flatten(&key, children, labels)?,
            Node::Other => return Err(LoadError::NotAString { key }),
        }
    }
    Ok(())
}
