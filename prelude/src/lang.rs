//! The language framework enabling multilingual output
//!
//!

use std::{error::Error, fmt::Display};

use anyhow::Result;
use Language::*;

/// List of supported languages
#[derive(Debug, Clone, Copy)]
pub enum Language {
    English,
    Hungarian,
}

/// Lists all languages (should be same order as the lang macro is used in)
fn langs() -> Vec<Language> {
    vec![Hungarian, English]
}

/// Default language
pub static mut LANGUAGE: Language = Hungarian;

/// Multilingual String macro, input &str's in the given order (currently Hungarian then English),
/// and this will return the appropriate String depending on the state of the LANGUAGE global variable
#[macro_export]
macro_rules! lang {
    ($hungarian:expr, $english:expr $(,)?) => {
        match unsafe { $crate::lang::LANGUAGE } {
            $crate::lang::Language::Hungarian => $hungarian.to_string(),
            $crate::lang::Language::English => $english.to_string(),
        }
    };
}

/// Sets the language to the input language
pub fn set_language(lang: Language) {
    unsafe {
        LANGUAGE = lang;
    }
}

/// Takes a string, and tries to find the closest matching language name.
/// Currently only works based off the English/given name of the language.
pub fn match_set_language(s: &str) -> Result<()> {
    let binding = langs();
    let lang: Vec<_> = binding
        .iter()
        .map(|l| {
            (
                format!("{:?}", l)
                    .to_lowercase()
                    .chars()
                    .filter(|c| s.to_lowercase().contains([*c]))
                    .count(),
                l,
            )
        })
        .collect();

    let max_match = lang.iter().max_by(|t1, t2| t1.0.cmp(&t2.0)).unwrap();

    if lang
        .iter()
        .filter(|(score, _)| score == &max_match.0)
        .count()
        != 1
    {
        return Err(NoLanguageMatchError {}.into());
    }

    set_language(*max_match.1);

    Ok(())
}

/// Safe abstraction to get the current language
pub fn get_language() -> Language {
    unsafe { LANGUAGE }
}

impl Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lang_name = match &self {
            Hungarian => lang!["Magyar", "Hungarian"],
            English => lang!["Angol", "English"],
        };

        write!(f, "{}", lang_name)
    }
}

#[derive(Debug)]
pub struct NoLanguageMatchError {}

impl Display for NoLanguageMatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe {
            write!(
                f,
                "Cannot match on language. Language will stay {:?}.",
                LANGUAGE
            )
        }
    }
}

impl Error for NoLanguageMatchError {}
