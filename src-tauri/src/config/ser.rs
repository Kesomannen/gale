use itertools::Itertools;
use serde::Serialize;

use super::{de::FLAGS_MESSAGE, Entry, File, FileMetadata, Num, Section, TaggedEntry, Value};
use std::io;

struct Serializer<W: io::Write> {
    writer: W,
}

impl<W: io::Write> Serializer<W> {
    #[inline]
    fn write(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.writer.write_all(bytes)
    }

    #[inline]
    fn new_line(&mut self) -> io::Result<()> {
        self.write(b"\n")
    }

    #[inline]
    fn write_str(&mut self, str: &str) -> io::Result<()> {
        self.write(str.as_bytes())
    }

    #[inline]
    fn write_into(&mut self, value: impl ToString) -> io::Result<()> {
        self.write_str(&value.to_string())
    }

    fn write_metadata(&mut self, metadata: &FileMetadata) -> io::Result<()> {
        self.write(b"## Settings file was created by plugin ")?;
        self.write_str(&metadata.plugin_name)?;
        self.write(b" ")?;
        self.write_str(&metadata.plugin_version)?;

        self.new_line()?;

        self.write(b"## Plugin GUID: ")?;
        self.write_str(&metadata.plugin_guid)?;

        self.new_line()?;
        self.new_line()?;

        Ok(())
    }

    fn write_section(&mut self, section: &Section) -> io::Result<()> {
        self.write(b"[")?;
        self.write_str(&section.name)?;
        self.write(b"]")?;
        self.new_line()?;
        self.new_line()?;

        for entry in section.entries.iter() {
            self.write_entry(entry)?;
            self.new_line()?;
        }

        Ok(())
    }

    fn write_num_comment<T>(&mut self, num: &Num<T>) -> io::Result<()>
    where
        T: Serialize + ToString,
    {
        if let Some(range) = &num.range {
            self.write(b"# Acceptable value range: From ")?;
            self.write_str(&range.start.to_string())?;
            self.write(b" to ")?;
            self.write_into(&range.end.to_string())?;
            self.new_line()?;
        }

        Ok(())
    }

    fn write_value(&mut self, value: &Value) -> io::Result<()> {
        match value {
            Value::Boolean(b) => self.write_into(b),
            Value::String(s) => self.write_str(s),
            Value::Enum { index, options } => self.write_str(&options[*index]),
            Value::Flags { indicies, options } => {
                self.write_str(&indicies.iter().map(|i| &options[*i]).join(", "))
            }
            Value::Int32(num) => self.write_into(num.value),
            Value::Single(num) => self.write_into(num.value),
            Value::Double(num) => self.write_into(num.value),
            Value::Other(s) => self.write_str(s),
        }
    }

    fn write_entry(&mut self, entry: &Entry) -> io::Result<()> {
        match entry {
            Entry::Tagged(tagged) => self.write_tagged_entry(tagged),
            Entry::Untagged { name, value } => self.write_untagged_entry(name, value),
        }
    }

    fn write_tagged_entry(&mut self, entry: &TaggedEntry) -> io::Result<()> {
        for line in entry.description.lines() {
            self.write(b"## ")?;
            self.write_str(line)?;
            self.new_line()?;
        }

        self.write(b"# Setting type: ")?;
        self.write_str(&entry.type_name)?;
        self.new_line()?;

        self.write(b"# Default value:")?;
        if let Some(default) = &entry.default_value {
            self.write(b" ")?;
            self.write_value(default)?;
        }
        self.new_line()?;

        if let Some(options) = entry.value.options() {
            self.write(b"# Acceptable values: ")?;
            let mut is_first = true;
            for option in options {
                if !is_first {
                    self.write(b", ")?;
                }
                is_first = false;

                self.write_str(option)?;
            }

            self.new_line()?;
        }

        if let Value::Flags { .. } = entry.value {
            self.write_str(FLAGS_MESSAGE)?;
            self.new_line()?;
        }

        match &entry.value {
            Value::Int32(num) => self.write_num_comment(num),
            Value::Single(num) => self.write_num_comment(num),
            Value::Double(num) => self.write_num_comment(num),
            _ => Ok(()),
        }?;

        self.write_str(&entry.name)?;
        self.write(b" = ")?;
        self.write_value(&entry.value)?;
        self.new_line()?;

        Ok(())
    }

    fn write_untagged_entry(&mut self, name: &str, value: &str) -> io::Result<()> {
        self.write_str(name)?;
        self.write(b" = ")?;
        self.write_str(value)?;
        self.new_line()?;

        Ok(())
    }
}

pub fn to_writer<W: io::Write>(file: &File, writer: W) -> io::Result<()> {
    let mut serializer = Serializer { writer };

    if let Some(metadata) = &file.metadata {
        serializer.write_metadata(metadata)?;
    }

    for section in file.sections.iter() {
        serializer.write_section(section)?;
    }

    serializer.writer.flush()
}

pub fn to_string(file: &File) -> io::Result<String> {
    let mut vec = Vec::new();
    to_writer(file, &mut vec)?;

    unsafe { Ok(String::from_utf8_unchecked(vec)) }
}
