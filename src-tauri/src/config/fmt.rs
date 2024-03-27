use super::{parser::FLAGS_MESSAGE, ConfigEntry, ConfigFile, ConfigValue};

impl ToString for ConfigValue {
    fn to_string(&self) -> String {
        match self {
            ConfigValue::Boolean(b) => b.to_string(),
            ConfigValue::String(s) => s.clone(),
            ConfigValue::Enum { value, .. } => value.clone(),
            ConfigValue::Flags { values, .. } => format!("{}\n{}", FLAGS_MESSAGE, values.join(", ")),
            ConfigValue::Int32(i) => i.value.to_string(),
            ConfigValue::Single(f) => f.value.to_string(),
            ConfigValue::Double(d) => d.value.to_string(),
            ConfigValue::Other { value, .. } => value.clone(),
        }
    }
}

impl ConfigValue {
    pub fn type_name(&self) -> &str {
        match self {
            ConfigValue::Boolean(_) => "Boolean",
            ConfigValue::String(_) => "String",
            ConfigValue::Enum { type_name, .. } => type_name,
            ConfigValue::Flags { type_name, .. } => type_name,
            ConfigValue::Int32(_) => "Int32",
            ConfigValue::Single(_) => "Single",
            ConfigValue::Double(_) => "Double",
            ConfigValue::Other { type_name, .. } => type_name,
        }
    }

    pub fn comment(&self) -> Option<String> {
        match self {
            ConfigValue::Enum { options, .. } => {
                Some(format!("# Acceptable values: {}", options.join(", ")))
            },
            ConfigValue::Flags { options, .. } => { 
                Some(format!("# Acceptable values: {}\n{}", options.join(", "), FLAGS_MESSAGE)) 
            },
            ConfigValue::Int32(range) => range.comment(),
            ConfigValue::Single(range) => range.comment(),
            ConfigValue::Double(range) => range.comment(),
            _ => None,
        }
    }
}

impl ToString for ConfigEntry {
    fn to_string(&self) -> String {
        match self {
            ConfigEntry::Config { name, description, default_value, value } => {
                let mut s = String::new();

                for line in description.lines() {
                    s.push_str("## ");
                    s.push_str(line);
                    s.push('\n');
                }

                s.push_str("# Setting type: ");
                s.push_str(value.type_name());

                s.push_str("\n# Default value: ");
                s.push_str(default_value.as_ref().unwrap_or(&String::new()).as_str());

                if let Some(comment) = value.comment() {
                    s.push('\n');
                    s.push_str(&comment);
                }

                s.push('\n');
                s.push_str(name);
                s.push_str(" = ");
                s.push_str(&value.to_string());

                s
            }
            ConfigEntry::Section { name, entries } => {
                let mut s = String::new();

                s.push_str("[");
                s.push_str(name);
                s.push_str("]");

                for entry in entries {
                    s.push_str("\n\n");
                    s.push_str(&entry.to_string());
                }

                s
            }
        }
    }
}

impl ToString for ConfigFile {
    fn to_string(&self) -> String {
        let mut s = String::new();

        for entry in &self.entries {
            s.push_str(&entry.to_string());
            s.push('\n');
        }

        s
    }
}