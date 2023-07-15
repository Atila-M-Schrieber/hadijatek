/* use itertools::Itertools;
use std::{error::Error, fmt::Display};

use eyre::Result;

pub trait FuzzyFindable {
    type Item;
    fn fzf_list(&self, s: Box<dyn Display>) -> (Self, usize)
    where
        Self: Sized;
}

impl FuzzyFindable for Vec<Box<dyn Display>> {
    type Item = Box<dyn Display>;
    fn fzf_list(&self, s: Box<dyn Display>) -> (Self, usize) {
        let s = format!("{}", s);
        let vec: Vec<_> = self
            .iter()
            .map(|l| {
                (
                    format!("{}", l)
                        .to_lowercase()
                        .chars()
                        .filter(|c| s.to_lowercase().contains([*c]))
                        .count(),
                    *l,
                )
            })
            .sorted_by_key(|t| t.0)
            .collect();

        let max_match = vec.iter().max_by(|t1, t2| t1.0.cmp(&t2.0)).unwrap();

        let num_max = vec
            .iter()
            .filter(|(score, _)| score == &max_match.0)
            .count();

        (vec.into_iter().map(|t| (*t.1)).collect(), num_max)
    }
}

#[derive(Debug)]
pub struct AmbiguousFuzzySearchResult(String);

impl Display for AmbiguousFuzzySearchResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cannot fuzzyly find {} in the provided text", self.0)
    }
}

impl Error for AmbiguousFuzzySearchResult {} */
