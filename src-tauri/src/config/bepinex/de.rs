use std::{
    fmt::Display,
    io::{BufRead, Lines, Read},
    str::{self, FromStr},
};

use eyre::{anyhow, bail, ensure, Context, OptionExt, Result};
use itertools::Itertools;
use serde::Serialize;

use super::*;

pub const FLAGS_MESSAGE: &str =
    "# Multiple values can be set at the same time by separating them with , (e.g. Debug, Warning)";

impl<T> Num<T>
where
    T: Serialize + FromStr + PartialOrd + Display,
    <T as FromStr>::Err: Send + Sync + std::error::Error + 'static,
{
    fn parse(value: &str, range: Option<&(String, String)>) -> Result<Self> {
        let value = value
            .replace(',', ".")
            .parse()
            .with_context(|| format!("failed to parse value '{}'", value))?;

        let range = match range {
            Some((min, max)) => {
                let min = min
                    .replace(',', ".")
                    .parse()
                    .with_context(|| format!("invalid minimum value '{}'", min))?;
                let max = max
                    .replace(',', ".")
                    .parse()
                    .with_context(|| format!("invalid maximum value '{}'", max))?;

                Some(min..max)
            }
            None => None,
        };

        Ok(Self { value, range })
    }
}

pub fn from_reader(reader: impl BufRead) -> Result<File> {
    let mut parser = Parser {
        lines: reader.lines(),
        peeked: None,
        line: 0,
        sections: Vec::new(),
        metadata: None,
    };

    match parser.parse() {
        Ok(_) => {
            let Parser {
                sections, metadata, ..
            } = parser;

            Ok(File { metadata, sections })
        }
        Err(err) => Err(err.wrap_err(format!("failed to parse file (at line {})", parser.line))),
    }
}

struct Parser<R: BufRead> {
    lines: Lines<R>,
    peeked: Option<String>,
    line: usize,
    sections: Vec<Section>,
    metadata: Option<Metadata>,
}

#[derive(Default)]
struct EntryBuilder {
    description: Option<String>,
    type_name: Option<String>,
    default_value: Option<String>,
    acceptable_values: Option<Vec<String>>,
    is_flags: bool,
    range: Option<(String, String)>,
    name: Option<String>,
    value: Option<String>,
}

impl EntryBuilder {
    fn build(self) -> Result<Entry> {
        let name = self.name.ok_or(anyhow!("missing entry name"))?;

        let type_name = self.type_name.ok_or(anyhow!("missing entry type"))?;

        let default_value = self
            .default_value
            .map(|string| {
                let options = self.acceptable_values.clone();

                Self::parse_value(
                    string,
                    options,
                    &type_name,
                    self.range.as_ref(),
                    self.is_flags,
                )
            })
            .transpose()?;

        let value = self.value.ok_or(anyhow!("missing entry value"))?;
        let value = Self::parse_value(
            value,
            self.acceptable_values,
            &type_name,
            self.range.as_ref(),
            self.is_flags,
        )?;

        Ok(Entry {
            name,
            type_name,
            default_value,
            value,
            description: self.description,
        })
    }

    fn parse_value(
        string: String,
        options: Option<Vec<String>>,
        type_name: &str,
        range: Option<&(String, String)>,
        is_flags: bool,
    ) -> Result<Value> {
        match options {
            Some(options) => Ok(Self::parse_enum(string, options, is_flags)),
            None => Self::parse_simple_value(string, type_name, range),
        }
    }

    fn parse_enum(string: String, options: Vec<String>, is_flags: bool) -> Value {
        match is_flags {
            true => Value::Flags {
                indicies: string
                    .split(", ")
                    .filter_map(|value| options.iter().position(|opt| opt == value))
                    .collect(),
                options,
            },
            false => Value::Enum {
                index: options
                    .iter()
                    .position(|opt| *opt == string)
                    .unwrap_or_default(),
                options,
            },
        }
    }

    fn parse_simple_value(
        value: String,
        type_name: &str,
        range: Option<&(String, String)>,
    ) -> Result<Value> {
        Ok(match type_name {
            "Boolean" => Value::Boolean(value.parse()?),
            "String" => Value::String(value.replace(r"\n", "\n")),
            "Int32" => Value::Int32(Num::parse(&value, range)?),
            "Single" => Value::Single(Num::parse(&value, range)?),
            "Double" => Value::Double(Num::parse(&value, range)?),
            _ => Value::Other(value),
        })
    }
}

impl<R: Read + BufRead> Parser<R> {
    fn parse(&mut self) -> Result<()> {
        while let Some(line) = self.peek()? {
            if line.is_empty() {
                self.consume()?;
                continue;
            }

            if line.starts_with('[') {
                self.parse_section()?;
            } else if line.starts_with('#') {
                if line.starts_with("## Settings file was created by plugin ") {
                    self.parse_metadata().ok();
                } else {
                    let entry = self.parse_entry()?;
                    self.push_entry(EntryKind::Normal(entry))?;
                }
            } else {
                let line = self.consume_or_eof()?;
                let (name, value) = self.parse_orphaned_entry(&line)?;

                let name = name.to_owned();
                let value = value.to_owned();

                self.push_entry(EntryKind::Orphaned { name, value })?;
            }
        }

        Ok(())
    }

    fn peek(&mut self) -> Result<Option<&str>> {
        if self.peeked.is_none() {
            self.peeked = self.next()?;
        }

        Ok(self.peeked.as_deref())
    }

    fn consume(&mut self) -> Result<Option<String>> {
        self.line += 1;
        match self.peeked.take() {
            Some(line) => Ok(Some(line)),
            None => self.next(),
        }
    }

    fn next(&mut self) -> Result<Option<String>> {
        let mut next = self.lines.next().transpose()?;

        if let Some(next_mut) = &mut next {
            // remove bom
            if self.line == 0 && next_mut.starts_with("\u{feff}") {
                next_mut.replace_range(0..3, "");
            }
        }

        Ok(next)
    }

    fn consume_or_eof(&mut self) -> Result<String> {
        self.consume()
            .and_then(|line| line.ok_or_eyre("unexpected end of file"))
    }

    fn parse_metadata(&mut self) -> Result<()> {
        let line = self.consume_or_eof()?;
        let mut split = line
            .strip_prefix("## Settings file was created by plugin ")
            .ok_or(anyhow!("expected metadata"))?
            .split(' ');

        let plugin_version = split
            .next_back()
            .ok_or(anyhow!("expected plugin version"))?
            .to_owned();

        let plugin_name = split.join(" ");

        let plugin_guid = self
            .consume_or_eof()?
            .strip_prefix("## Plugin GUID: ")
            .ok_or(anyhow!("expected plugin GUID"))?
            .to_owned();

        self.metadata = Some(Metadata {
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

    fn parse_multiline_comment(&mut self, prefix: &str) -> Result<String> {
        let mut buffer = String::new();

        while let Some(line) = self.peek()? {
            if let Some(line) = line.strip_prefix(prefix) {
                if !buffer.is_empty() {
                    buffer.push('\n');
                }

                buffer.push_str(line.trim());
                self.consume()?;
            } else {
                break;
            }
        }

        Ok(buffer)
    }

    fn parse_entry(&mut self) -> Result<Entry> {
        let description = self.parse_multiline_comment("##")?;
        let mut builder = EntryBuilder {
            description: Some(description),
            ..Default::default()
        };

        loop {
            let line = self.consume_or_eof()?;

            if line == FLAGS_MESSAGE {
                builder.is_flags = true;
            } else if let Some(line) = line.strip_prefix("# ") {
                if let Some(type_name) = line.strip_prefix("Setting type: ") {
                    builder.type_name = Some(type_name.to_owned());
                } else if let Some(default_value) = line.strip_prefix("Default value: ") {
                    builder.default_value = Some(default_value.to_owned());
                } else if let Some(acceptable_values) = line.strip_prefix("Acceptable values: ") {
                    builder.acceptable_values =
                        Some(acceptable_values.split(", ").map(str::to_owned).collect());
                } else if let Some(range) = line.strip_prefix("Acceptable value range: From ") {
                    let (min, max) = range
                        .split_once(" to ")
                        .ok_or_eyre("expected value range")?;
                    builder.range = Some((min.to_owned(), max.to_owned()));
                }
            } else {
                let (name, value) = self.parse_orphaned_entry(&line)?;
                builder.name = Some(name.to_owned());
                builder.value = Some(value.to_owned());
                break;
            }
        }

        builder.build()
    }

    fn parse_orphaned_entry<'a>(&mut self, line: &'a str) -> Result<(&'a str, &'a str)> {
        line.split_once("=")
            .ok_or(anyhow!("expected entry name"))
            .map(|(name, value)| (name.trim(), value.trim()))
    }

    fn push_entry(&mut self, entry: EntryKind) -> Result<()> {
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
