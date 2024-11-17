use std::io::{BufRead, Write};

use eyre::{bail, eyre, OptionExt, Result};
use indexmap::IndexMap;
use serde_json::{Number, Value};

use super::frontend::{self, Num};

#[derive(Debug)]
pub struct File {
    entries: IndexMap<String, Value>,
}

impl File {
    pub fn read(reader: impl BufRead) -> Result<Self> {
        let entries = serde_json::from_reader(reader)?;
        Ok(File { entries })
    }

    pub fn write(&self, writer: impl Write) -> Result<()> {
        serde_json::to_writer_pretty(writer, &self.entries)?;
        Ok(())
    }

    pub fn set(&mut self, name: impl Into<String>, value: frontend::Value) -> Result<()> {
        self.entries.insert(name.into(), value.try_into()?);
        Ok(())
    }

    pub fn to_frontend(&self) -> Result<frontend::FileData> {
        let entries = self
            .entries
            .iter()
            .map(|(key, value)| {
                let name = key.to_owned();
                let value = value.to_owned().try_into()?;

                Ok(frontend::Entry {
                    name,
                    value,
                    description: None,
                    default: None,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(frontend::FileData {
            metadata: None,
            sections: vec![frontend::Section {
                name: "Default".into(),
                entries,
            }],
        })
    }
}

impl TryFrom<Value> for frontend::Value {
    type Error = eyre::Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Bool(bool) => Ok(frontend::Value::Bool(bool)),
            Value::String(str) => Ok(frontend::Value::String(str)),
            Value::Number(number) => {
                if let Some(num) = number.as_i64() {
                    return Ok(frontend::Value::Int(Num {
                        value: num as i32,
                        range: None,
                    }));
                }

                if let Some(num) = number.as_f64() {
                    return Ok(frontend::Value::Float(Num {
                        value: num as f32,
                        range: None,
                    }));
                }

                Err(eyre!("unsupported number value"))
            }
            value => bail!("unsupported JSON value type: {}", value),
        }
    }
}

impl TryFrom<frontend::Value> for Value {
    type Error = eyre::Error;

    fn try_from(value: frontend::Value) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            frontend::Value::Bool(bool) => Value::Bool(bool),
            frontend::Value::String(str) => Value::String(str),
            frontend::Value::Int(num) => Value::Number(num.value.into()),
            frontend::Value::Float(num) => {
                let value = Number::from_f64(num.value as f64)
                    .ok_or_eyre("cannot serialize NaN or infinite value")?;
                Value::Number(value)
            }
            _ => bail!("unsupported config value"),
        })
    }
}
