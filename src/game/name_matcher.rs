//! Fuzzy matching of the miniature names people write in stock sheets ("Nebulon B Frigate",
//! "Tie Fighter") against the builder models (`nebulon-b`, `tie-fighter`).
//!
//! An exact title wins outright. Otherwise each word of the name scores for the models that
//! contain it, weighted by how rare the word is among the candidates, so "Interdictor Class
//! Destroyer" picks `interdictor` over the many `*-class-*-destroyer` models.

use std::collections::{BTreeSet, HashMap};

use crate::game::model_catalogue::CatalogueModel;

/// Score of a word that only prefixes a model word ("mc30" for "mc30c"), relative to an exact one.
const PREFIX_WEIGHT: f64 = 0.5;
/// Shortest word allowed to match by prefix, so "a" or "mk" match nothing.
const MIN_PREFIX_LEN: usize = 3;
/// Bonus when the name contains every word of the builder type ("Tie Fighter", `tie-fighter`).
const FULL_ID_BONUS: f64 = 0.5;
/// Scores closer than this are a tie.
const EPSILON: f64 = 1e-9;

/// Splits `text` into lowercase alphanumeric words: "GR-75 Medium" → `gr`, `75`, `medium`.
pub(crate) fn tokens(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Returns the candidate `name` designates, or `None` if no candidate shares a word with it
/// or the two best ones tie.
pub(crate) fn best_match<'a>(
    name: &str,
    candidates: &[&'a CatalogueModel],
) -> Option<&'a CatalogueModel> {
    let words = tokens(name);
    if words.is_empty() {
        return None;
    }
    if let Some(model) = exact_match(&words, candidates) {
        return Some(model);
    }
    let frequency = word_frequency(candidates);
    let mut scored: Vec<(f64, &CatalogueModel)> = candidates
        .iter()
        .map(|model| (score(&words, model, &frequency), *model))
        .filter(|(score, _)| *score > EPSILON)
        .collect();
    scored.sort_by(|a, b| b.0.total_cmp(&a.0));
    match scored.as_slice() {
        [(best, model), (second, _), ..] if best - second > EPSILON => Some(model),
        [(_, model)] => Some(model),
        _ => None,
    }
}

/// The single candidate whose type or one of whose titles is `words`, if any.
fn exact_match<'a>(
    words: &[String],
    candidates: &[&'a CatalogueModel],
) -> Option<&'a CatalogueModel> {
    let mut matching = candidates.iter().filter(|model| {
        tokens(&model.id) == words || model.titles.iter().any(|title| tokens(title) == words)
    });
    let first = matching.next()?;
    matching.next().is_none().then_some(*first)
}

/// Number of candidates containing each word.
fn word_frequency<'a>(candidates: &[&'a CatalogueModel]) -> HashMap<&'a str, usize> {
    let mut frequency = HashMap::new();
    for model in candidates {
        for word in &model.words {
            *frequency.entry(word.as_str()).or_insert(0) += 1;
        }
    }
    frequency
}

fn score(words: &[String], model: &CatalogueModel, frequency: &HashMap<&str, usize>) -> f64 {
    let rarity = |word: &str| 1.0 / frequency.get(word).copied().unwrap_or(1).max(1) as f64;
    let mut total: f64 = words
        .iter()
        .map(|word| {
            if model.words.contains(word) {
                return rarity(word);
            }
            prefix_partners(word, &model.words)
                .map(|partner| PREFIX_WEIGHT * rarity(partner))
                .fold(0.0, f64::max)
        })
        .sum();
    let id_words: BTreeSet<String> = tokens(&model.id).into_iter().collect();
    if id_words.iter().all(|word| words.contains(word)) {
        total += FULL_ID_BONUS;
    }
    total
}

/// Model words that `word` prefixes or is prefixed by, both being long enough.
fn prefix_partners<'a>(
    word: &'a str,
    model_words: &'a BTreeSet<String>,
) -> impl Iterator<Item = &'a str> {
    model_words.iter().map(String::as_str).filter(move |other| {
        word.len().min(other.len()) >= MIN_PREFIX_LEN
            && (other.starts_with(word) || word.starts_with(other))
    })
}
