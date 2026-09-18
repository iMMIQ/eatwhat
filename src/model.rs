use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error(pub String);
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { self.0.fmt(f) }
}
impl std::error::Error for Error {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub id: String,
    pub title: String,
    pub license: String,
    pub reference: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Dish {
    pub id: String,
    pub name: String,
    #[serde(default)] pub aliases: Vec<String>,
    /// Exactly one primary sampling category prevents double counting.
    pub category: String,
    #[serde(default)] pub regions: Vec<String>,
    #[serde(default)] pub tags: Vec<String>,
    #[serde(default)] pub ingredients: Vec<String>,
    #[serde(default)] pub ingredients_complete: bool,
    #[serde(default)] pub meals: Vec<String>,
    #[serde(default)] pub modes: Vec<String>,
    #[serde(default)] pub themes: Vec<String>,
    /// Conservative upper estimate; unknown is excluded when a time limit is supplied.
    pub minutes_max: Option<u32>,
    /// Ingredient cost per serving in CNY fen, not a restaurant price.
    pub cost_max_fen: Option<u32>,
    /// Editorial familiarity prior in [0,1], not observed popularity.
    pub familiarity: f64,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Strategy { Uniform, #[default] Balanced, Comfort, Explore, Surprise }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Options {
    pub count: usize,
    pub seed: Option<u64>,
    pub strategy: Strategy,
    pub categories: Vec<String>,
    pub regions: Vec<String>,
    pub meal: Option<String>,
    pub mode: Option<String>,
    pub theme: Option<String>,
    pub max_minutes: Option<u32>,
    pub budget_max_fen: Option<u32>,
    pub exclude_ingredients: Vec<String>,
    /// Fail closed on incomplete ingredient lists when excluding ingredients.
    pub require_complete_ingredients: bool,
    pub required_tags: Vec<String>,
    pub preferences: Vec<String>,
    /// IDs, names or aliases. Earlier positions are more recent.
    pub recent: Vec<String>,
    pub exclude_recent: bool,
    /// Replaces the built-in pack. None selects the built-in catalog.
    pub catalog: Option<crate::Catalog>,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            count: 1, seed: None, strategy: Strategy::Balanced,
            categories: vec![], regions: vec![], meal: None, mode: None, theme: None,
            max_minutes: None, budget_max_fen: None, exclude_ingredients: vec![],
            require_complete_ingredients: true, required_tags: vec![], preferences: vec![],
            recent: vec![], exclude_recent: true, catalog: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Recommendation { pub dish: Dish, pub reason_codes: Vec<String> }
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Response {
    pub items: Vec<Recommendation>,
    pub status: String,
    pub candidate_count: usize,
    pub requested_count: usize,
    pub seed: u64,
    pub dataset_id: String,
    pub dataset_version: String,
    pub algorithm_version: String,
    pub warnings: Vec<String>,
}
