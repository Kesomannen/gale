use std::{iter, str};

use anyhow::{bail, ensure, Context, Result};

use super::*;

pub const FLAGS_MESSAGE: &'static str =
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

                ensure!(
                    min <= max,
                    "minimum value '{}' is greater than maximum value '{}'",
                    min,
                    max
                );
                ensure!(
                    value >= min && value <= max,
                    "value '{}' is out of range [{}, {}]",
                    value,
                    min,
                    max
                );

                Some(min..max)
            }
            None => None,
        };

        Ok(Self { value, range })
    }
}

pub fn from_str(text: &str) -> Result<Vec<Section>> {
    let mut parser = Parser {
        lines: text.lines().peekable(),
        sections: Vec::new(),
        line: 1,
    };

    parser
        .parse()
        .with_context(|| format!("error parsing config file at line {}", parser.line))?;

    Ok(parser.sections)
}

struct Parser<'a> {
    lines: iter::Peekable<str::Lines<'a>>,
    sections: Vec<Section>,
    line: usize,
}

impl<'a> Parser<'a> {
    fn parse(&mut self) -> Result<()> {
        while let Some(line) = self.peek() {
            if line.is_empty() {
                self.consume();
                continue;
            }

            if line.starts_with("[") {
                self.parse_section()?;
            } else if line.starts_with('#') {
                if let Some(entry) = self.parse_entry()? {
                    ensure!(!self.sections.is_empty(), "config entry has no section");

                    let section = self.sections.last_mut().unwrap();
                    section.entries.push(entry);
                }
            } else {
                bail!("unexpected line '{}'", line);
            }
        }

        Ok(())
    }

    fn peek(&mut self) -> Option<&&'a str> {
        self.lines.peek()
    }

    fn peek_or_eof(&mut self) -> Result<&&'a str> {
        self.peek().context("unexpected EOF")
    }

    fn consume(&mut self) -> Option<&'a str> {
        self.line += 1;
        self.lines.next()
    }

    fn consume_or_eof(&mut self) -> Result<&'a str> {
        self.consume().context("unexpected EOF")
    }

    fn parse_section(&mut self) -> Result<()> {
        let line = self.consume_or_eof()?;

        if !line.starts_with("[") || !line.ends_with("]") {
            bail!("expected section header, found '{}'", line);
        }

        let name = &line[1..line.len() - 1];
        self.sections.push(Section {
            name: name.to_owned(),
            entries: Vec::new(),
        });

        Ok(())
    }

    fn parse_entry(&mut self) -> Result<Option<Entry>> {
        let mut description = String::new();

        while let Some(line) = self.peek() {
            if line.starts_with("##") {
                if !description.is_empty() {
                    description.push('\n');
                }

                description.push_str(&line[2..].trim());
                self.consume();
            } else {
                break;
            }
        }

        let line = self.consume();
        if line.is_none() {
            return Ok(None);
        }

        let line = line.unwrap();

        let type_name = match line.strip_prefix("# Setting type: ") {
            Some(type_id) => type_id,
            None => return Ok(None),
        };

        let line = self.consume_or_eof()?;
        let default_value = line
            .strip_prefix("# Default value: ")
            .with_context(|| format!("expected default value, found '{}'", line))?
            .trim();

        let default_value_str = match default_value.is_empty() {
            true => None,
            false => Some(default_value),
        };

        let line = self.peek_or_eof()?;
        let acceptable_values = line
            .strip_prefix("# Acceptable values: ")
            .map(|s| s.split(", ").map(|s| s.to_owned()).collect::<Vec<_>>());

        let mut is_flags = false;

        if acceptable_values.is_some() {
            self.consume();
            let line = self.peek_or_eof()?;
            if *line == FLAGS_MESSAGE {
                self.consume();
                is_flags = true;
            }
        }

        let line = self.peek_or_eof()?;
        let range = match line.strip_prefix("# Acceptable value range: From ") {
            Some(s) => {
                self.consume();
                let mut split = s.split(" to ");
                Some((
                    split.next().context("expected minimum value")?,
                    split.next().context("expected maximum value")?,
                ))
            }
            None => None,
        };

        let mut split = self.consume_or_eof()?.split(" = ");
        let name = split.next().context("expected setting name")?;
        let value_str = split.next().context("expected setting value")?;

        let (value, default_value) = parse_value(
            acceptable_values,
            is_flags,
            value_str,
            default_value_str,
            type_name,
            range,
        )?;

        Ok(Some(Entry {
            name: name.to_owned(),
            type_name: type_name.to_owned(),
            description,
            default_value,
            value,
        }))
    }
}

fn parse_value(
    acceptable_values: Option<Vec<String>>,
    is_flags: bool,
    value_str: &str,
    default_value_str: Option<&str>,
    type_name: &str,
    range: Option<(&str, &str)>,
) -> Result<(Value, Option<Value>)> {
    Ok(match acceptable_values {
        Some(options) => match is_flags {
            true => (
                Value::Flags {
                    values: parse_flags(value_str),
                    options: options.clone(),
                },
                default_value_str.map(|s| Value::Flags {
                    values: parse_flags(s),
                    options,
                }),
            ),
            false => (
                Value::Enum {
                    value: value_str.to_owned(),
                    options: options.clone(),
                },
                default_value_str.map(|s| Value::Enum {
                    value: s.to_owned(),
                    options,
                }),
            ),
        },
        None => (
            parse_simple_value(type_name, value_str, range)?,
            default_value_str
                .map(|s| parse_simple_value(type_name, s, range))
                .transpose()?,
        ),
    })
}

fn parse_simple_value(
    type_name: &str,
    value_str: &str,
    range: Option<(&str, &str)>,
) -> Result<Value> {
    Ok(match type_name {
        "Boolean" => Value::Boolean(value_str == "true"),
        "String" => Value::String(value_str.to_owned()),
        "Int32" => Value::Int32(Num::parse(value_str, range)?),
        "Single" => Value::Single(Num::parse(value_str, range)?),
        "Double" => Value::Double(Num::parse(value_str, range)?),
        _ => Value::Other(value_str.to_string()),
    })
}

fn parse_flags(value_str: &str) -> Vec<String> {
    value_str.split(", ").map(|s| s.to_owned()).collect()
}
