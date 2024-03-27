use std::{iter, str};

use anyhow::{bail, Context, Result};

use super::*;

pub const FLAGS_MESSAGE: &'static str = "# Multiple values can be set at the same time by separating them with , (e.g. Debug, Warning)";

pub fn parse(text: &str) -> Result<Vec<ConfigEntry>> {
    let mut parser = Parser {
        lines: text.lines().peekable(),
        current_section: None,
        entries: Vec::new(),
        line: 0,
    };

    parser.parse().with_context(|| format!("error parsing config file at line {}", parser.line))?;

    Ok(parser.entries)
}

struct Parser<'a> {
    lines: iter::Peekable<str::Lines<'a>>,
    current_section: Option<&'a str>,
    entries: Vec<ConfigEntry>,
    line: usize,
}

impl<'a> Parser<'a> {
    fn parse(&mut self) -> Result<()> {
        while let Some(line) = self.peek() {
            if line.is_empty() {
                continue;
            }
            
            if line.starts_with("[") {
                self.parse_section()?;
            } else if line.starts_with('#') {
                let entry = self.parse_entry()?;
                self.add_entry(entry);
            }
        }

        Ok(())
    }

    fn peek(&mut self) -> Option<&'a str> {
        self.lines.peek().map(|s| s.trim())
    }

    fn peek_or_eof(&mut self) -> Result<&'a str> {
        self.peek().context("unexpected EOF")
    }

    fn consume(&mut self) -> Option<&'a str> {
        self.line += 1;
        self.lines.next().map(str::trim)
    }

    fn consume_or_eof(&mut self) -> Result<&'a str> {
        self.consume().context("unexpected EOF")
    }

    fn parse_section(&mut self) -> Result<()> {
        let line = self.consume_or_eof()?;

        if !line.starts_with("[") || !line.ends_with("]") {
            bail!("expected section header, found '{}'", line);
        }

        let section = &line[1..line.len() - 1];
        self.current_section = Some(section);

        Ok(())
    }

    fn parse_entry(&mut self) -> Result<ConfigEntry> {
        let mut description = String::new();

        while let Some(line) = self.peek() {
            if line.starts_with("##") {
                description.push('\n');
                description.push_str(&line[2..].trim());
                self.consume();
            } else {
                break;
            }
        }

        let line = self.consume_or_eof()?;
        let type_id = line.strip_prefix("# Setting type: ")
            .with_context(|| format!("expected setting type, found {}", line))?;

        let line = self.consume_or_eof()?;
        let default_value = line.strip_prefix("# Default value: ")
            .with_context(|| format!("expected default value, found {}", line))?;

        let line = self.peek_or_eof()?;
        let acceptable_values = line.strip_prefix("# Acceptable values: ")
            .map(|s| s.split(", ").map(|s| s.to_string()).collect::<Vec<_>>());

        let mut is_flags = false;

        if acceptable_values.is_some() {
            self.consume();
            let line = self.peek_or_eof()?;
            if line == FLAGS_MESSAGE {
                self.consume();
                is_flags = true;
            }
        }

        let mut split = self.consume_or_eof()?.split(" = ");
        let name = split.next().context("expected setting name")?;
        let value_str = split.next().context("expected setting value")?;

        let value = {
            if let Some(values) = acceptable_values {
                match is_flags {
                    true => ConfigValue::Flags { 
                        values: value_str.split(", ").map(|s| s.to_string()).collect(),
                        options: values,
                        type_name: type_id.to_string(),
                    },
                    false => ConfigValue::Enum {
                        value: value_str.to_string(),
                        options: values,
                        type_name: type_id.to_string(),
                    },
                }
            } else {
                match type_id {
                    "Boolean" => ConfigValue::Boolean(value_str == "true"),
                    "String" => ConfigValue::String(value_str.to_string()),
                    "Int32" => ConfigValue::Int32(value_str.parse().context("expected integer value")?),
                    "Single" => ConfigValue::Single(value_str.parse().context("expected float value")?),
                    "Double" => ConfigValue::Double(value_str.parse().context("expected double value")?),
                    _ => ConfigValue::Other {
                        type_name: type_id.to_string(),
                        value: value_str.to_string(),
                    },
                }
            }
        };

        Ok(ConfigEntry::Config {
            name: name.to_string(),
            description,
            default_value: default_value.to_string(),
            value,
        })
    }
    
    fn add_entry(&mut self, entry: ConfigEntry) {
        if let Some(full_section_name) = self.current_section {
            full_section_name
                .split('.')
                .fold(&mut self.entries, search)
                .push(entry);
        } else {
            self.entries.push(entry);
        }
        
        fn search<'a>(entries: &'a mut Vec<ConfigEntry>, section_name: &str) -> &'a mut Vec<ConfigEntry> {
            let index = entries.iter().position(|e| match e {
                ConfigEntry::Section { name, .. } => name == section_name,
                _ => false,
            });

            match index {
                Some(index) => {
                    let section = entries.get_mut(index).unwrap();

                    match section {
                        ConfigEntry::Section { entries, .. } => entries,
                        _ => unreachable!(),
                    }
                },
                None => {
                    let new_section = ConfigEntry::Section {
                        name: section_name.to_string(),
                        entries: Vec::new(),
                    };
        
                    entries.push(new_section);
        
                    match entries.last_mut().unwrap() {
                        ConfigEntry::Section { entries, .. } => entries,
                        _ => unreachable!(),
                    }
                },
            }
        }
    }
}
