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
    #[cfg(target_os = "linux")]
    {
        shell_words::join(words)
    }

    #[cfg(target_os = "windows")]
    {
        use itertools::Itertools;

        words
            .into_iter()
            .map(|s| {
                // escape and quote the argument
                let s = s.as_ref();

                if s.contains(' ') || s.contains('"') {
                    let escaped = s.replace('"', "\\\"");
                    format!("\"{escaped}\"")
                } else {
                    s.to_string()
                }
            })
            .join(" ")
    }
}

fn split(custom_args: &str) -> Result<Vec<String>> {
    #[cfg(target_os = "linux")]
    {
        shell_words::split(custom_args).context("failed to split arguments")
    }

    #[cfg(target_os = "windows")]
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
    prefix: Option<Vec<String>>,
}

impl CustomArgs {
    fn apply(&self, command: &mut Command) {
        for (key, value) in &self.env {
            command.env(key, value);
        }

        if let Some([prefix_cmd, prefix_args @ ..]) = self.prefix.as_deref() {
            let mut new_command = Command::new(prefix_cmd);
            new_command.args(prefix_args);
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
                prefix = Some(args);
                args = Vec::new();
            } else if let Some((key, value)) = word.split_once('=').filter(|(k, _)| is_env_name(k)) {
                env.push((key.to_string(), value.to_string()));
            } else {
                args.push(word);
            }
        }

        Ok(Self { args, env, prefix })
    }
}

fn is_env_name(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(chars.next(), Some(c) if c == '_' || c.is_ascii_alphabetic())
        && chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    fn new_args(args: Vec<&str>, env: Vec<(&str, &str)>, prefix: Option<Vec<&str>>) -> CustomArgs {
        CustomArgs {
            args: args.into_iter().map(String::from).collect(),
            env: env
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            prefix: prefix.map(|v| v.iter().map(|s| s.to_string()).collect()),
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
        let expected = new_args(vec!["--foo"], vec![], Some(vec!["protontricks"]));
        assert_eq!(result, expected);
    }

    #[test]
    fn prefix_multi() {
        let result =
            CustomArgs::from_str("env -u DISPLAY mangohud --fps-limit=60 %command% --foo").unwrap();
        let expected = new_args(
            vec!["--foo"],
            vec![],
            Some(vec!["env", "-u", "DISPLAY", "mangohud", "--fps-limit=60"]),
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn multiple_prefixes_fail() {
        let result = CustomArgs::from_str("protontricks %command% anotherlauncher %command% --foo");
        assert!(result.is_err());
    }

    #[test]
    fn multiple_empty_prefixes_fail() {
        let result = CustomArgs::from_str("%command% %command% --foo");
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
            Some(vec!["protontricks"]),
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn complex_prefix_multi() {
        let result = CustomArgs::from_str(
            r#"FOO=bar BAZ="qux quux" env -u DISPLAY mangohud --fps-limit=60 %command% --foo "bar baz" "something else""#,
        )
        .unwrap();
        let expected = new_args(
            vec!["--foo", "bar baz", "something else"],
            vec![("FOO", "bar"), ("BAZ", "qux quux")],
            Some(vec!["env", "-u", "DISPLAY", "mangohud", "--fps-limit=60"]),
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn apply() {
        let custom_args = new_args(
            vec!["--foo", "bar baz"],
            vec![("FOO", "bar")],
            Some(vec!["protontricks"]),
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

    #[test]
    fn apply_prefix_multi() {
        let custom_args = new_args(
            vec!["--foo", "bar baz"],
            vec![("FOO", "bar")],
            Some(vec!["env", "-u", "DISPLAY", "mangohud", "--fps-limit=60"]),
        );

        let mut command = Command::new("game_executable --original-arg");
        custom_args.apply(&mut command);

        println!("{command:#?}\n{custom_args:#?}");

        let mut expected = Command::new("env");
        expected
            .args(["-u", "DISPLAY", "mangohud", "--fps-limit=60"])
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

    #[test]
    #[cfg(target_os = "linux")]
    fn join_linux() {
        let args = vec!["--foo", "bar baz", "something else"];
        let joined = join(args);
        assert_eq!(joined, "--foo 'bar baz' 'something else'");
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn join_windows() {
        let args = vec![
            "--foo",
            "bar baz",
            "something else",
            r"C:\A\Path\With Spaces",
        ];
        let joined = join(args);
        assert_eq!(
            joined,
            r#"--foo "bar baz" "something else" "C:\A\Path\With Spaces""#
        );
    }
}
