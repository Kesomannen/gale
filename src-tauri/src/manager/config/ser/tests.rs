use super::*;

impl File {
    fn new(name: &str, sections: Vec<Section>) -> Self {
        Self {
            name: name.to_owned(),
            sections,
        }
    }
}

impl Section {
    fn new(name: &str, entries: Vec<Entry>) -> Self {
        Self {
            name: name.to_owned(),
            entries,
        }
    }
}

impl Entry {
    fn new(name: &str, description: &str, default_value: Option<Value>, value: Value) -> Self {
        let type_name = match &value {
            Value::Boolean(_) => "Boolean",
            Value::String(_) => "String",
            Value::Int32(_) => "Int32",
            Value::Single(_) => "Single",
            Value::Double(_) => "Double",
            _ => panic!("cannot determine type name"),
        };

        Self::new_typed(name, description, type_name, default_value, value)
    }

    fn new_typed(
        name: &str,
        description: &str,
        type_name: &str,
        default_value: Option<Value>,
        value: Value,
    ) -> Entry {
        Self {
            name: name.to_owned(),
            description: description.to_owned(),
            type_name: type_name.to_owned(),
            default_value,
            value,
        }
    }
}

#[test]
fn test_to_string() {
    let file = File::new(
        "test",
        vec![
            Section::new(
                "Section1",
                vec![
                    Entry::new(
                        "Entry1",
                        "This is entry 1",
                        Some(Value::String("Default".to_owned())),
                        Value::String("Value1".to_owned()),
                    ),
                    Entry::new_typed(
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
                    Entry::new_typed(
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
                    Entry::new(
                        "Entry4",
                        "This is entry 4",
                        None,
                        Value::Int32(Num {
                            value: 5,
                            range: Some(0..10),
                        }),
                    ),
                    Entry::new(
                        "Entry5",
                        "This is entry 5",
                        Some(Value::Double(Num {
                            value: 2.0,
                            range: None,
                        })),
                        Value::Double(Num {
                            value: 3.14,
                            range: None,
                        }),
                    ),
                ],
            ),
        ],
    );

    let expected = r###"[Section1]

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
Entry5 = 3.14

"###;

    assert_eq!(to_string(&file), expected);
}
