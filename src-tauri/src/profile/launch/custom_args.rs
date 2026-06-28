use eyre::{Context, Result, bail};
use std::{fmt::Display, process::Command, str::FromStr};

pub fn add_args(command: &mut Command, custom_args: &str) -> Result<()> {
    let args: CustomArgs = custom_args.parse().context("failed to parse custom args")?;

    args.apply(command);

    Ok(())
}

pub fn join<I, S>(words: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<str> + Display,
{
    #[cfg(unix)]
    {
        shell_words::join(words)
    }

    #[cfg(not(unix))]
    {
        use itertools::Itertools;

        words.into_iter().join(" ")
    }
}

fn split(custom_args: &str) -> Result<Vec<String>> {
    #[cfg(unix)]
    {
        shell_words::split(custom_args).context("failed to split arguments")
    }

    #[cfg(not(unix))]
    {
        Ok(winsplit::split(custom_args))
    }
}

/// A parsed set of custom arguments:
///
/// - `args`: list of command-line arguments to append to the launch command
///
/// - `env`: list of environment variables to set for the launch command,
///   written as VARIABLE=value before the rest of the arguments
///
/// - `prefix`: an optional prefix to prepend to the launch command,
///   which can be used to run the game with a custom launcher (e.g. `protontricks`),
///   written using a %command% placeholder
#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct CustomArgs {
    args: Vec<String>,
    env: Vec<(String, String)>,
    prefix: Option<String>,
}

impl CustomArgs {
    fn apply(&self, command: &mut Command) {
        for (key, value) in &self.env {
            command.env(key, value);
        }

        if let Some(prefix) = &self.prefix {
            let mut new_command = Command::new(prefix);
            new_command.arg(command.get_program());
            new_command.args(command.get_args());
            for (key, value) in command.get_envs() {
                match value {
                    Some(val) => new_command.env(key, val),
                    None => new_command.env_remove(key),
                };
            }

            *command = new_command;
        }

        command.args(&self.args);
    }
}

impl FromStr for CustomArgs {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self> {
        let words = split(s)?;

        let mut args = Vec::new();
        let mut env = Vec::new();
        let mut prefix = None;

        for word in words {
            if word == "%command%" {
                if prefix.is_some() {
                    bail!("multiple %command% placeholders are not allowed");
                }
                prefix = Some(args.join(" "));
                args.clear();
            } else if let Some((key, value)) = word.split_once('=') {
                env.push((key.to_string(), value.to_string()));
            } else {
                args.push(word);
            }
        }

        Ok(Self { args, env, prefix })
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    fn new_args(args: Vec<&str>, env: Vec<(&str, &str)>, prefix: Option<&str>) -> CustomArgs {
        CustomArgs {
            args: args.into_iter().map(String::from).collect(),
            env: env
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            prefix: prefix.map(String::from),
        }
    }

    #[test]
    fn empty() {
        let result = CustomArgs::from_str("").unwrap();
        let expected = CustomArgs::default();
        assert_eq!(result, expected);
    }

    #[test]
    fn simple_args() {
        let result = CustomArgs::from_str("--foo bar").unwrap();
        let expected = new_args(vec!["--foo", "bar"], vec![], None);
        assert_eq!(result, expected);
    }

    #[test]
    fn quoted_args() {
        let result = CustomArgs::from_str(r#"--foo "bar baz""#).unwrap();
        let expected = new_args(vec!["--foo", "bar baz"], vec![], None);
        assert_eq!(result, expected);
    }

    #[test]
    fn unix_quoted_args() {
        let result = CustomArgs::from_str(r#"--name "Mac Profile""#).unwrap();
        assert_eq!(result.args, vec!["--name", "Mac Profile"]);
    }

    #[test]
    fn env_var() {
        let result = CustomArgs::from_str("FOO=bar WINE=yes --baz qux").unwrap();
        let expected = new_args(
            vec!["--baz", "qux"],
            vec![("FOO", "bar"), ("WINE", "yes")],
            None,
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn quoted_env_vars() {
        let result = CustomArgs::from_str(r#"FOO="bar baz" --qux quux"#).unwrap();
        let expected = new_args(vec!["--qux", "quux"], vec![("FOO", "bar baz")], None);
        assert_eq!(result, expected);
    }

    #[test]
    fn prefix() {
        let result = CustomArgs::from_str("protontricks %command% --foo").unwrap();
        let expected = new_args(vec!["--foo"], vec![], Some("protontricks"));
        assert_eq!(result, expected);
    }

    #[test]
    fn multiple_prefixes_fail() {
        let result = CustomArgs::from_str("protontricks %command% anotherlauncher %command% --foo");
        assert!(result.is_err());
    }

    #[test]
    fn complex() {
        let result = CustomArgs::from_str(
            r#"FOO=bar BAZ="qux quux" protontricks %command% --foo "bar baz" "something else""#,
        )
        .unwrap();
        let expected = new_args(
            vec!["--foo", "bar baz", "something else"],
            vec![("FOO", "bar"), ("BAZ", "qux quux")],
            Some("protontricks"),
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn apply() {
        let custom_args = new_args(
            vec!["--foo", "bar baz"],
            vec![("FOO", "bar")],
            Some("protontricks"),
        );

        let mut command = Command::new("game_executable --original-arg");
        custom_args.apply(&mut command);

        println!("{command:#?}\n{custom_args:#?}");

        let mut expected = Command::new("protontricks");
        expected
            .arg("game_executable --original-arg")
            .args(["--foo", "bar baz"])
            .env("FOO", "bar");

        assert_eq!(command.get_program(), expected.get_program());
        assert_eq!(
            command.get_args().collect_vec(),
            expected.get_args().collect_vec()
        );
        assert_eq!(
            command.get_envs().collect_vec(),
            expected.get_envs().collect_vec()
        );
    }
}
