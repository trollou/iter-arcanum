//! Analyse d'un libellé : texte littéral et paramètres `{nom}`, `{{` et `}}` littéraux.

use std::collections::BTreeSet;

use crate::{LoadError, is_valid_name};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Part {
    Text(String),
    Parameter(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Label {
    pub(crate) parts: Vec<Part>,
}

impl Label {
    pub(crate) fn parse(key: &str, text: &str) -> Result<Self, LoadError> {
        if text.is_empty() {
            return Err(LoadError::EmptyLabel {
                key: key.to_owned(),
            });
        }
        let malformed = || LoadError::MalformedBraces {
            key: key.to_owned(),
        };
        let mut parts = Vec::new();
        let mut literal = String::new();
        let mut chars = text.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                '{' if chars.next_if_eq(&'{').is_some() => literal.push('{'),
                '}' if chars.next_if_eq(&'}').is_some() => literal.push('}'),
                '}' => return Err(malformed()),
                '{' => {
                    let mut name = String::new();
                    loop {
                        match chars.next() {
                            Some('}') => break,
                            Some(c) => name.push(c),
                            None => return Err(malformed()),
                        }
                    }
                    if name.is_empty() {
                        return Err(malformed());
                    }
                    if !is_valid_name(&name) {
                        return Err(LoadError::InvalidParameterName {
                            key: key.to_owned(),
                            parameter: name,
                        });
                    }
                    if !literal.is_empty() {
                        parts.push(Part::Text(std::mem::take(&mut literal)));
                    }
                    parts.push(Part::Parameter(name));
                }
                c => literal.push(c),
            }
        }
        if !literal.is_empty() {
            parts.push(Part::Text(literal));
        }
        Ok(Self { parts })
    }

    pub(crate) fn parameters(&self) -> BTreeSet<String> {
        self.parts
            .iter()
            .filter_map(|part| match part {
                Part::Parameter(name) => Some(name.clone()),
                Part::Text(_) => None,
            })
            .collect()
    }
}
