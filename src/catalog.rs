use crate::{Dish, Error, Source};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    pub schema_version: u32,
    pub id: String,
    pub version: String,
    pub sources: Vec<Source>,
    pub dishes: Vec<Dish>,
}

pub(crate) fn norm(s: &str) -> String { s.trim().to_lowercase() }
fn nonempty(s: &str) -> bool { !s.trim().is_empty() }
impl Catalog {
    pub fn from_json(text: &str) -> Result<Self, Error> {
        let catalog: Self = serde_json::from_str(text).map_err(|e| Error(e.to_string()))?;
        catalog.validate()?;
        Ok(catalog)
    }
    pub fn validate(&self) -> Result<(), Error> {
        if self.schema_version != 1 || !nonempty(&self.id) || !nonempty(&self.version) {
            return Err(Error("invalid catalog header (schema_version must be 1)".into()));
        }
        let mut sources = BTreeSet::new();
        for s in &self.sources {
            if [&s.id, &s.title, &s.license, &s.reference].iter().any(|v| !nonempty(v))
                || !sources.insert(s.id.as_str()) {
                return Err(Error("invalid or duplicate source".into()));
            }
        }
        let mut ids = BTreeSet::new();
        let mut names: BTreeMap<String, &str> = BTreeMap::new();
        for d in &self.dishes {
            if !nonempty(&d.id) || !nonempty(&d.name) || !nonempty(&d.category)
                || !ids.insert(d.id.as_str()) || !d.familiarity.is_finite()
                || !(0.0..=1.0).contains(&d.familiarity)
                || d.sources.is_empty() || d.sources.iter().any(|s| !sources.contains(s.as_str()))
                || (d.ingredients_complete && d.ingredients.is_empty()) {
                return Err(Error(format!("invalid dish: {}", d.id)));
            }
            for list in [&d.aliases, &d.regions, &d.tags, &d.ingredients, &d.meals, &d.modes, &d.themes] {
                if list.iter().any(|v| !nonempty(v)) {
                    return Err(Error(format!("empty label in {}", d.id)));
                }
            }
            for name in std::iter::once(&d.name).chain(d.aliases.iter()).chain(std::iter::once(&d.id)) {
                if let Some(other) = names.insert(norm(name), &d.id) {
                    if other != d.id { return Err(Error(format!("ambiguous name/alias/id: {name}"))); }
                }
            }
        }
        Ok(())
    }
    pub(crate) fn resolve(&self, query: &str) -> Option<&Dish> {
        let query = norm(query);
        self.dishes.iter().find(|d| norm(&d.id) == query || norm(&d.name) == query
            || d.aliases.iter().any(|a| norm(a) == query))
    }
}
