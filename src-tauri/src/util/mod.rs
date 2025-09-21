use std::cmp::Ordering;

use itertools::Itertools;

pub mod cmd;
pub mod error;
pub mod fs;
pub mod path;
pub mod vdf_parser;
pub mod window;
pub mod zip;

pub fn cmp_ignore_case(a: impl AsRef<str>, b: impl AsRef<str>) -> Ordering {
    a.as_ref()
        .chars()
        .flat_map(char::to_lowercase)
        .zip_longest(b.as_ref().chars().flat_map(char::to_lowercase))
        .map(|ab| match ab {
            itertools::EitherOrBoth::Left(_) => Ordering::Greater,
            itertools::EitherOrBoth::Right(_) => Ordering::Less,
            itertools::EitherOrBoth::Both(a, b) => a.cmp(&b),
        })
        .find(|&ordering| ordering != Ordering::Equal)
        .unwrap_or(Ordering::Equal)
}
