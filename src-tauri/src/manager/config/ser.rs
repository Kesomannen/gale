use serde::Serialize;

use super::{de::FLAGS_MESSAGE, Entry, File, FileMetadata, Num, Section, TaggedEntry, Value};

struct Serializer {
    buffer: String,
}

impl Serializer {
    fn push<T: ToString>(&mut self, value: &T) {
        self.buffer.push_str(&value.to_string());
    }

    fn push_char(&mut self, c: char) {
        self.buffer.push(c);
    }

    fn push_str(&mut self, s: &str) {
        self.buffer.push_str(s);
    }

    fn new_line(&mut self) {
        self.buffer.push('\n');
    }

    fn push_line(&mut self, line: &str) {
        self.buffer.push_str(line);
        self.new_line();
    }

    fn write_metadata(&mut self, metadata: &FileMetadata) {
        self.push_line(&format!("## Settings file was created by plugin {} {}", metadata.plugin_name, metadata.plugin_version));
        self.push_line(&format!("## Plugin GUID: {}", metadata.plugin_guid));
        self.new_line();
    }

    fn write_section(&mut self, section: &Section) {
        self.push_char('[');
        self.push_str(&section.name);
        self.push_line("]");
        self.new_line();

        for entry in section.entries.iter() {
            self.write_entry(entry);
            self.new_line();
        }
    }

    fn write_num_comment<T>(&mut self, num: &Num<T>)
    where
        T: Serialize + ToString,
    {
        if let Some(range) = &num.range {
            self.push_str("# Acceptable value range: From ");
            self.push(&range.start);
            self.push_str(" to ");
            self.push(&range.end);
            self.new_line();
        }
    }

    fn write_value(&mut self, value: &Value) {
        match value {
            Value::Boolean(b) => self.push(&b),
            Value::String(s) => self.push_str(s),
            Value::Enum { value, .. } => self.push_str(value),
            Value::Flags { values, .. } => self.push_str(&values.join(", ")),
            Value::Int32(num) => self.push(&num.value),
            Value::Single(num) => self.push(&num.value),
            Value::Double(num) => self.push(&num.value),
            Value::Other(s) => self.push_str(s),
        };
    }

    fn write_entry(&mut self, entry: &Entry) {
        match entry {
            Entry::Tagged(tagged) => self.write_tagged_entry(tagged),
            Entry::Untagged { name, value } => self.write_untagged_entry(name, value),
        }
    }

    fn write_tagged_entry(&mut self, entry: &TaggedEntry) {
        for line in entry.description.lines() {
            self.push_str("## ");
            self.push_line(line);
        }

        self.push_str("# Setting type: ");
        self.push_line(&entry.type_name);

        self.push_str("# Default value:");
        if let Some(default) = &entry.default_value {
            self.push_char(' ');
            self.write_value(default);
        }
        self.new_line();

        if let Some(options) = entry.value.options() {
            self.push_str("# Acceptable values: ");
            let mut is_first = true;
            for option in options {
                if !is_first {
                    self.push_str(", ");
                }
                is_first = false;

                self.push_str(option);
            }

            self.new_line();
        }

        if let Value::Flags { .. } = entry.value {
            self.push_line(FLAGS_MESSAGE);
        }

        match &entry.value {
            Value::Int32(num) => self.write_num_comment(num),
            Value::Single(num) => self.write_num_comment(num),
            Value::Double(num) => self.write_num_comment(num),
            _ => {}
        }

        self.push_str(&entry.name);
        self.push_str(" = ");
        self.write_value(&entry.value);
        self.new_line();
    }

    fn write_untagged_entry(&mut self, name: &str, value: &str) {
        self.push_str(name);
        self.push_str(" = ");
        self.push_str(value);
        self.new_line();
    }
}

pub fn to_string(file: &File) -> String {
    let mut serializer = Serializer {
        buffer: String::new(),
    };

    if let Some(metadata) = &file.metadata {
        serializer.write_metadata(metadata);
    }

    for section in file.sections.iter() {
        serializer.write_section(section);
    }

    serializer.buffer
}
