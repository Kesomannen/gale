use std::{iter, str};

use anyhow::{bail, ensure, Context, Result};

use super::*;
use itertools::Itertools;

pub const FLAGS_MESSAGE: &str =
    "# Multiple values can be set at the same time by separating them with , (e.g. Debug, Warning)";

impl<T> Num<T>
where
    T: Serialize + FromStr + PartialOrd + Display,
    <T as FromStr>::Err: Send + Sync + std::error::Error + 'static,
{
    fn parse(value: &str, range: Option<(&str, &str)>) -> Result<Self> {
        let value = value
            .parse()
            .with_context(|| format!("failed to parse value '{}'", value))?;

        let range = match range {
            Some((min, max)) => {
                let min = min
                    .parse()
                    .with_context(|| format!("invalid minimum value '{}'", min))?;
                let max = max
                    .parse()
                    .with_context(|| format!("invalid maximum value '{}'", max))?;

                Some(min..max)
            }
            None => None,
        };

        Ok(Self { value, range })
    }
}

pub fn from_str(text: &str) -> Result<(Vec<Section>, Option<FileMetadata>)> {
    let mut parser = Parser {
        lines: text.lines().peekable(),
        sections: Vec::new(),
        metadata: None,
        line: 0,
    };

    match parser.parse() {
        Ok(_) => Ok((parser.sections, parser.metadata)),
        Err(err) => Err(err.context(format!("error parsing config file at line {}", parser.line))),
    }
}

struct Parser<'a> {
    lines: iter::Peekable<str::Lines<'a>>,
    sections: Vec<Section>,
    metadata: Option<FileMetadata>,
    line: usize,
}

#[derive(Default)]
struct EntryBuilder<'a> {
    description: Option<String>,
    type_name: Option<&'a str>,
    default_value: Option<&'a str>,
    acceptable_values: Option<Vec<&'a str>>,
    is_flags: bool,
    range: Option<(&'a str, &'a str)>,
    name: Option<&'a str>,
    value: Option<&'a str>,
}

impl EntryBuilder<'_> {
    fn build(self) -> Result<TaggedEntry> {
        let name = self.name.context("missing entry name")?.to_owned();
        let type_name = self.type_name.context("missing entry type")?.to_owned();

        let value = self.value.context("missing entry value")?;
        let value = self.parse_value(value)?;

        let default_value = self
            .default_value
            .map(|s| self.parse_value(s))
            .transpose()?;
        let description = self.description.unwrap_or_default();

        Ok(TaggedEntry {
            name,
            description,
            type_name,
            default_value,
            value,
        })
    }

    fn parse_value(&self, str: &str) -> Result<Value> {
        match &self.acceptable_values {
            Some(options) => Ok(self.parse_enum(str, options)),
            None => self.parse_simple_value(str),
        }
    }

    fn parse_enum(&self, str: &str, options: &[&str]) -> Value {
        let options = options.iter().map(|s| (*s).to_owned()).collect_vec();

        match self.is_flags {
            true => Value::Flags {
                indicies: str
                    .split(", ")
                    .filter_map(|value| options.iter().position(|opt| opt == value))
                    .collect(),
                options,
            },
            false => Value::Enum {
                index: options
                    .iter()
                    .position(|opt| opt == str)
                    .unwrap_or_default(),
                options,
            },
        }
    }

    fn parse_simple_value(&self, value: &str) -> Result<Value> {
        Ok(match self.type_name.context("missing entry type")? {
            "Boolean" => Value::Boolean(value.parse()?),
            "String" => Value::String(value.to_owned()),
            "Int32" => Value::Int32(Num::parse(value, self.range)?),
            "Single" => Value::Single(Num::parse(value, self.range)?),
            "Double" => Value::Double(Num::parse(value, self.range)?),
            _ => Value::Other(value.to_owned()),
        })
    }
}

impl<'a> Parser<'a> {
    fn parse(&mut self) -> Result<()> {
        while let Some(line) = self.peek() {
            if line.is_empty() {
                self.consume();
                continue;
            }

            if line.starts_with('[') {
                self.parse_section()?;
            } else if line.starts_with('#') {
                if line.starts_with("## Settings file was created by plugin ") {
                    self.parse_metadata().ok();
                } else {
                    let entry = self.parse_tagged_entry()?;
                    self.push_entry(Entry::Tagged(entry))?;
                }
            } else {
                let (name, value) = self.parse_untagged_entry()?;
                let name = name.to_owned();
                let value = value.to_owned();

                self.push_entry(Entry::Untagged { name, value })?;
            }
        }

        Ok(())
    }

    fn peek(&mut self) -> Option<&&'a str> {
        self.lines.peek()
    }

    fn consume(&mut self) -> Option<&'a str> {
        self.line += 1;
        self.lines.next()
    }

    fn consume_or_eof(&mut self) -> Result<&'a str> {
        self.consume().context("unexpected EOF")
    }

    fn parse_metadata(&mut self) -> Result<()> {
        let mut split = self
            .consume_or_eof()?
            .strip_prefix("## Settings file was created by plugin ")
            .context("expected metadata")?
            .split(' ');

        let plugin_version = split
            .next_back()
            .context("expected plugin version")?
            .to_owned();

        let plugin_name = split.join(" ");

        let plugin_guid = self
            .consume_or_eof()?
            .strip_prefix("## Plugin GUID: ")
            .context("expected plugin GUID")?
            .to_owned();

        self.metadata = Some(FileMetadata {
            plugin_name,
            plugin_version,
            plugin_guid,
        });

        Ok(())
    }

    fn parse_section(&mut self) -> Result<()> {
        let line = self.consume_or_eof()?;

        if !line.starts_with('[') || !line.ends_with(']') {
            bail!("expected section header, found '{}'", line);
        }

        let name = &line[1..line.len() - 1];
        self.sections.push(Section {
            name: name.to_owned(),
            entries: Vec::new(),
        });

        Ok(())
    }

    fn parse_multiline_comment(&mut self, prefix: &str) -> String {
        let mut buffer = String::new();

        while let Some(line) = self.peek() {
            if let Some(line) = line.strip_prefix(prefix) {
                if !buffer.is_empty() {
                    buffer.push('\n');
                }

                buffer.push_str(line.trim());
                self.consume();
            } else {
                break;
            }
        }

        buffer
    }

    fn parse_tagged_entry(&mut self) -> Result<TaggedEntry> {
        let mut builder = EntryBuilder {
            description: Some(self.parse_multiline_comment("##")),
            ..Default::default()
        };

        loop {
            let line = self.peek().context("unexpected EOF")?;

            if line == &FLAGS_MESSAGE {
                self.consume();

                builder.is_flags = true;
            } else if let Some(line) = line.strip_prefix("# ") {
                self.consume();

                if let Some(type_name) = line.strip_prefix("Setting type: ") {
                    builder.type_name = Some(type_name);
                } else if let Some(default_value) = line.strip_prefix("Default value: ") {
                    builder.default_value = Some(default_value);
                } else if let Some(acceptable_values) = line.strip_prefix("Acceptable values: ") {
                    builder.acceptable_values = Some(acceptable_values.split(", ").collect());
                } else if let Some(range) = line.strip_prefix("Acceptable value range: From ") {
                    let mut split = range.split(" to ");
                    builder.range = Some((
                        split.next().context("expected minimum value")?,
                        split.next().context("expected maximum value")?,
                    ));
                }
            } else {
                let (name, value) = self.parse_untagged_entry()?;
                builder.name = Some(name);
                builder.value = Some(value);
                break;
            }
        }

        builder.build()
    }

    fn parse_untagged_entry(&mut self) -> Result<(&str, &str)> {
        let mut split = self.consume_or_eof()?.split(" = ");
        let name = split.next().context("expected entry name")?;
        let value_str = split.next().unwrap_or_default();

        Ok((name, value_str))
    }

    fn push_entry(&mut self, entry: Entry) -> Result<()> {
        ensure!(
            !self.sections.is_empty(),
            "entry {} has no section",
            entry.name()
        );

        let section = self.sections.last_mut().unwrap();
        section.entries.push(entry);

        Ok(())
    }
}
