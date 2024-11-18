use std::io::{BufRead, Write};

use eyre::{eyre, OptionExt, Result};

use super::frontend::{self, Num};

pub mod de;
pub mod ser;

#[cfg(test)]
mod tests;

#[derive(Debug, PartialEq)]
pub struct File {
    metadata: Option<Metadata>,
    sections: Vec<Section>,
}

impl File {
    pub fn mod_name(&self) -> Option<&str> {
        self.metadata
            .as_ref()
            .map(|metadata| metadata.plugin_name.as_str())
    }

    pub fn read(reader: impl BufRead) -> Result<Self> {
        de::from_reader(reader)
    }

    pub fn write(&self, writer: impl Write) -> Result<()> {
        ser::to_writer(self, writer)?;
        Ok(())
    }

    pub fn to_frontend(&self) -> frontend::FileData {
        let metadata = self.metadata.as_ref().map(|metadata| frontend::Metadata {
            mod_name: metadata.plugin_name.clone(),
            mod_version: metadata.plugin_version.clone(),
        });

        let sections = self.sections.iter().map(Section::to_frontend).collect();

        frontend::FileData { metadata, sections }
    }

    fn find_section(&mut self, name: &str) -> Result<&mut Section> {
        self.sections
            .iter_mut()
            .find(|section| section.name == name)
            .ok_or_eyre("section not found")
    }

    pub fn find_entry(&mut self, section: &str, entry: &str) -> Result<&mut EntryKind> {
        self.find_section(section)
            .and_then(|section| section.find_entry(entry))
    }
}

#[derive(Debug, PartialEq)]
pub struct Metadata {
    plugin_name: String,
    plugin_version: String,
    plugin_guid: String,
}

#[derive(Debug, PartialEq)]
pub struct Section {
    name: String,
    entries: Vec<EntryKind>,
}

impl Section {
    fn to_frontend(&self) -> frontend::Section {
        let entries = self
            .entries
            .iter()
            .filter_map(EntryKind::to_frontend)
            .collect();

        frontend::Section {
            name: self.name.clone(),
            entries,
        }
    }

    fn find_entry(&mut self, name: &str) -> Result<&mut EntryKind> {
        self.entries
            .iter_mut()
            .find(|entry| entry.name() == name)
            .ok_or_eyre("entry not found")
    }
}

#[derive(Debug, PartialEq)]
pub enum EntryKind {
    Normal(Entry),
    Orphaned { name: String, value: String },
}

impl EntryKind {
    fn name(&self) -> &str {
        match self {
            Self::Normal(e) => &e.name,
            Self::Orphaned { name, .. } => name,
        }
    }

    fn as_normal_mut(&mut self) -> Result<&mut Entry> {
        match self {
            Self::Normal(e) => Ok(e),
            Self::Orphaned { .. } => Err(eyre!("entry is not tagged")),
        }
    }

    fn to_frontend(&self) -> Option<frontend::Entry> {
        match self {
            Self::Normal(entry) => Some(entry.to_frontend()),
            Self::Orphaned { .. } => None,
        }
    }

    pub fn set(&mut self, value: frontend::Value) -> Result<()> {
        self.as_normal_mut()?.value = value.into();
        Ok(())
    }

    pub fn reset(&mut self) -> Result<frontend::Value> {
        self.as_normal_mut()?.reset()
    }
}

impl From<Entry> for EntryKind {
    fn from(e: Entry) -> Self {
        Self::Normal(e)
    }
}

#[derive(Debug, PartialEq)]
pub struct Entry {
    name: String,
    description: Option<String>,
    type_name: String,
    default_value: Option<Value>,
    value: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Boolean(bool),
    String(String),
    Int32(Num<i32>),
    Single(Num<f32>),
    Double(Num<f64>),
    Other(String),
    Enum {
        index: usize,
        options: Vec<String>,
    },
    Flags {
        indicies: Vec<usize>,
        options: Vec<String>,
    },
}

impl Entry {
    fn reset(&mut self) -> Result<frontend::Value> {
        self.value = self.default_value.clone().ok_or_eyre("no default value")?;
        Ok(self.value.clone().into())
    }

    fn to_frontend(&self) -> frontend::Entry {
        frontend::Entry {
            name: self.name.clone(),
            description: self.description.clone(),
            default: self.default_value.clone().map(|value| value.into()),
            value: self.value.clone().into(),
        }
    }
}

impl Value {
    fn options(&self) -> Option<&[String]> {
        match self {
            Self::Enum { options, .. } => Some(options),
            Self::Flags { options, .. } => Some(options),
            _ => None,
        }
    }
}

impl From<Value> for frontend::Value {
    fn from(value: Value) -> Self {
        match value {
            Value::Boolean(bool) => frontend::Value::Bool(bool),
            Value::String(str) => frontend::Value::String(str),
            Value::Int32(num) => frontend::Value::Int(num),
            Value::Single(num) => frontend::Value::Float(num),
            Value::Double(num) => frontend::Value::Float(Num {
                value: num.value as f32,
                range: num
                    .range
                    .map(|range| (range.start as f32)..(range.end as f32)),
            }),
            Value::Other(str) => frontend::Value::String(str),
            Value::Enum { index, options } => frontend::Value::Enum { index, options },
            Value::Flags { indicies, options } => frontend::Value::Flags { indicies, options },
        }
    }
}

impl From<frontend::Value> for Value {
    fn from(value: frontend::Value) -> Self {
        match value {
            frontend::Value::Bool(bool) => Value::Boolean(bool),
            frontend::Value::String(str) => Value::String(str),
            frontend::Value::Int(num) => Value::Int32(num),
            frontend::Value::Float(num) => Value::Single(num),
            frontend::Value::Enum { index, options } => Value::Enum { index, options },
            frontend::Value::Flags { indicies, options } => Value::Flags { indicies, options },
        }
    }
}
