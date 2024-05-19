use super::*;

impl Section {
    fn new(name: &str, entries: Vec<Entry>) -> Self {
        Self {
            name: name.to_owned(),
            entries,
        }
    }
}

impl Entry {
    fn untagged(name: &str, value: &str) -> Self {
        Self::Untagged {
            name: name.to_owned(),
            value: value.to_owned(),
        }
    }

    fn tagged(name: &str, description: &str, default_value: Option<Value>, value: Value) -> Self {
        let type_name = match &value {
            Value::Boolean(_) => "Boolean",
            Value::String(_) => "String",
            Value::Int32(_) => "Int32",
            Value::Single(_) => "Single",
            Value::Double(_) => "Double",
            _ => panic!("cannot determine type name"),
        };

        Self::tagged_typed(name, description, type_name, default_value, value)
    }

    fn tagged_typed(
        name: &str,
        description: &str,
        type_name: &str,
        default_value: Option<Value>,
        value: Value,
    ) -> Self {
        TaggedEntry {
            name: name.to_owned(),
            description: description.to_owned(),
            type_name: type_name.to_owned(),
            default_value,
            value,
        }.into()
    }
}

const TEST_STR: &str = r###"## Settings file was created by plugin Plugin v1.0.0
## Plugin GUID: Author.PluginGuid

[Section1]

## This is entry 1
# Setting type: String
# Default value: Default
Entry1 = Value1

## This is entry 2
# Setting type: LogLevel
# Default value: Info, Warning, Error
# Acceptable values: Debug, Info, Warning, Error
# Multiple values can be set at the same time by separating them with , (e.g. Debug, Warning)
LogLevels = Info, Warning

## This is entry 3
# Setting type: Difficulty
# Default value: Medium
# Acceptable values: Easy, Medium, Hard
Entry3 = Easy

[Section2]

## This is entry 4
# Setting type: Int32
# Default value:
# Acceptable value range: From 0 to 10
Entry4 = 5

## This is entry 5
# Setting type: Double
# Default value: 2
Entry5 = 3.13

UntaggedEntry = Hi!

"###;

fn test_file() -> File {
    File::new(
        "test".to_owned(),
        vec![
            Section::new(
                "Section1",
                vec![
                    Entry::tagged(
                        "Entry1",
                        "This is entry 1",
                        Some(Value::String("Default".to_owned())),
                        Value::String("Value1".to_owned()),
                    ),
                    Entry::tagged_typed(
                        "LogLevels",
                        "This is entry 2",
                        "LogLevel",
                        Some(Value::Flags {
                            values: vec![
                                "Info".to_owned(),
                                "Warning".to_owned(),
                                "Error".to_owned(),
                            ],
                            options: vec![
                                "Debug".to_owned(),
                                "Info".to_owned(),
                                "Warning".to_owned(),
                                "Error".to_owned(),
                            ],
                        }),
                        Value::Flags {
                            values: vec!["Info".to_owned(), "Warning".to_owned()],
                            options: vec![
                                "Debug".to_owned(),
                                "Info".to_owned(),
                                "Warning".to_owned(),
                                "Error".to_owned(),
                            ],
                        },
                    ),
                    Entry::tagged_typed(
                        "Entry3",
                        "This is entry 3",
                        "Difficulty",
                        Some(Value::Enum {
                            value: "Medium".to_owned(),
                            options: vec![
                                "Easy".to_owned(),
                                "Medium".to_owned(),
                                "Hard".to_owned(),
                            ],
                        }),
                        Value::Enum {
                            value: "Easy".to_owned(),
                            options: vec![
                                "Easy".to_owned(),
                                "Medium".to_owned(),
                                "Hard".to_owned(),
                            ],
                        },
                    ),
                ],
            ),
            Section::new(
                "Section2",
                vec![
                    Entry::tagged(
                        "Entry4",
                        "This is entry 4",
                        None,
                        Value::Int32(Num {
                            value: 5,
                            range: Some(0..10),
                        }),
                    ),
                    Entry::tagged(
                        "Entry5",
                        "This is entry 5",
                        Some(Value::Double(Num {
                            value: 2.0,
                            range: None,
                        })),
                        Value::Double(Num {
                            value: 3.13,
                            range: None,
                        }),
                    ),
                    Entry::untagged("UntaggedEntry", "Hi!"),
                ],
            ),
        ],
        Some(FileMetadata { 
            plugin_name: "Plugin".to_owned(),
            plugin_version: "1.0.0".to_owned(),
            plugin_guid: "Author.PluginGuid".to_owned(),
        })
    )
}

#[test]
fn test_to_string() {
    assert_eq!(ser::to_string(&test_file()), TEST_STR);
}

#[test]
fn test_from_string() {
    let (sections, metadata) = de::from_str(TEST_STR).unwrap();
    assert_eq!(File::new("test".to_owned(), sections, metadata), test_file());
}
