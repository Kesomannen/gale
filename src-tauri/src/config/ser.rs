use serde::Serialize;

use super::{de::FLAGS_MESSAGE, Entry, EntryKind, File, FileMetadata, Num, Section, Value};
use std::{
    fmt::Display,
    io::{self, Write},
};

struct Serializer<W: Write> {
    writer: W,
}

impl<W: Write> Write for Serializer<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<W: Write> Serializer<W> {
    fn write_metadata(&mut self, metadata: &FileMetadata) -> io::Result<()> {
        writeln!(
            self,
            "## Settings file was created by plugin {} {}",
            metadata.plugin_name, metadata.plugin_version
        )?;
        writeln!(self, "## Plugin GUID: {}", metadata.plugin_guid)?;
        writeln!(self)?;

        Ok(())
    }

    fn write_section(&mut self, section: &Section) -> io::Result<()> {
        writeln!(self, "[{}]", section.name)?;
        writeln!(self)?;

        for entry in section.entries.iter() {
            self.write_entry_kind(entry)?;
        }

        Ok(())
    }

    fn write_num_comment<T>(&mut self, num: &Num<T>) -> io::Result<()>
    where
        T: Serialize + Display,
    {
        if let Some(range) = &num.range {
            writeln!(
                self,
                "# Acceptable value range: From {} to {}",
                range.start, range.end
            )?;
        }

        Ok(())
    }

    fn write_value(&mut self, value: &Value) -> io::Result<()> {
        match value {
            Value::String(str) => write!(self, "{}", str.replace('\n', r"\n")),
            Value::Other(str) => write!(self, "{str}"),
            Value::Boolean(bool) => write!(self, "{bool}"),
            Value::Int32(num) => write!(self, "{}", num.value),
            Value::Single(num) => write!(self, "{}", num.value),
            Value::Double(num) => write!(self, "{}", num.value),
            Value::Enum { index, options } => write!(self, "{}", options[*index]),
            Value::Flags { indicies, options } => {
                if indicies.is_empty() {
                    return write!(self, "0");
                }

                for (i, option) in indicies.iter().map(|index| &options[*index]).enumerate() {
                    if i > 0 {
                        write!(self, ", ")?;
                    }

                    write!(self, "{option}")?;
                }

                Ok(())
            }
        }
    }

    fn write_entry_kind(&mut self, entry: &EntryKind) -> io::Result<()> {
        match entry {
            EntryKind::Normal(entry) => self.write_entry(entry),
            EntryKind::Orphaned { name, value } => self.write_orphaned_entry(name, value),
        }
    }

    fn write_entry(&mut self, entry: &Entry) -> io::Result<()> {
        if let Some(description) = &entry.description {
            for line in description.lines() {
                writeln!(self, "## {}", line)?;
            }
        }

        writeln!(self, "# Setting type: {}", entry.type_name)?;

        write!(self, "# Default value:")?;
        if let Some(default) = &entry.default_value {
            write!(self, " ")?;
            self.write_value(default)?;
        }
        writeln!(self)?;

        if let Some(options) = entry.value.options() {
            write!(self, "# Acceptable values: ")?;
            for (i, option) in options.iter().enumerate() {
                if i > 0 {
                    write!(self, ", ")?;
                }

                write!(self, "{option}")?;
            }

            writeln!(self)?;
        }

        if let Value::Flags { .. } = entry.value {
            writeln!(self, "{}", FLAGS_MESSAGE)?;
        }

        match &entry.value {
            Value::Int32(num) => self.write_num_comment(num),
            Value::Single(num) => self.write_num_comment(num),
            Value::Double(num) => self.write_num_comment(num),
            _ => Ok(()),
        }?;

        write!(self, "{} = ", entry.name)?;
        self.write_value(&entry.value)?;
        writeln!(self)?;
        writeln!(self)?;

        Ok(())
    }

    fn write_orphaned_entry(&mut self, name: &str, value: &str) -> io::Result<()> {
        writeln!(self, "{name} = {value}")?;
        writeln!(self)?;

        Ok(())
    }
}

pub fn to_writer<W: Write>(file: &File, writer: W) -> io::Result<()> {
    let mut serializer = Serializer { writer };

    if let Some(metadata) = &file.metadata {
        serializer.write_metadata(metadata)?;
    }

    for section in file.sections.iter() {
        serializer.write_section(section)?;
    }

    serializer.writer.flush()
}

#[allow(unused)]
pub fn to_string(file: &File) -> io::Result<String> {
    let mut vec = Vec::new();
    to_writer(file, &mut vec)?;

    unsafe { Ok(String::from_utf8_unchecked(vec)) }
}
