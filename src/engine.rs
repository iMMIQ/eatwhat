use crate::catalog::norm;
use crate::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::{collections::{BTreeMap, BTreeSet}, sync::OnceLock};

static BUILTIN: OnceLock<Catalog> = OnceLock::new();
fn has(values: &[String], value: &str) -> bool { values.iter().any(|v| norm(v) == norm(value)) }
fn intersects(a: &[String], b: &[String]) -> bool { a.iter().any(|v| has(b, v)) }
fn similarity(a: &Dish, b: &Dish) -> f64 {
    if a.id == b.id { return 1.0; }
    let category = if norm(&a.category) == norm(&b.category) { 0.5 } else { 0.0 };
    let ingredients = if intersects(&a.ingredients, &b.ingredients) { 0.3 } else { 0.0 };
    let tags = if intersects(&a.tags, &b.tags) { 0.2 } else { 0.0 };
    category + ingredients + tags
}
fn weight(d: &Dish, o: &Options, history: &[&Dish], chosen: &[&Dish]) -> f64 {
    if o.strategy == Strategy::Uniform { return 1.0; }
    let preference_hits = o.preferences.iter().map(|s| norm(s)).collect::<BTreeSet<_>>()
        .iter().filter(|s| has(&d.tags, s) || has(&d.ingredients, s)).count();
    let familiarity = match o.strategy {
        Strategy::Comfort => 0.25 + 2.0 * d.familiarity,
        Strategy::Explore => 1.25 - d.familiarity,
        Strategy::Surprise => 1.5 - d.familiarity,
        _ => 1.0,
    };
    let repeated = history.iter().enumerate()
        .map(|(i, h)| similarity(d, h) / (1.0 + i as f64 * 0.25))
        .fold(0.0_f64, f64::max);
    let batch_similarity = chosen.iter().map(|h| similarity(d, h)).fold(0.0_f64, f64::max);
    let history_strength = if o.strategy == Strategy::Explore { 0.9 } else { 0.7 };
    (1.0 + preference_hits as f64 * 0.75) * familiarity
        * (1.0 - repeated * history_strength) * (1.0 - batch_similarity * 0.6)
}
fn weighted_index(weights: &[f64], rng: &mut ChaCha8Rng) -> usize {
    let total: f64 = weights.iter().sum();
    let mut target = rng.gen::<f64>() * total;
    for (i, w) in weights.iter().enumerate() {
        if target < *w { return i; }
        target -= w;
    }
    weights.len() - 1
}

/// Recommend without replacement. Hard filters never relax. Empty matches are a normal response.
/// Seed reproducibility requires the same catalog contents, options and algorithm version.
pub fn recommend(o: &Options) -> Result<Response, Error> {
    if !(1..=100).contains(&o.count) { return Err(Error("count must be in 1..=100".into())); }
    if o.budget_max_fen.is_some() && o.mode.as_deref() != Some("cook") {
        return Err(Error("budget_max_fen is ingredient cost; mode must be cook".into()));
    }
    for list in [&o.categories, &o.regions, &o.exclude_ingredients, &o.required_tags, &o.preferences, &o.recent] {
        if list.iter().any(|s| s.trim().is_empty()) { return Err(Error("empty filter label".into())); }
    }
    for s in [&o.meal, &o.mode, &o.theme].into_iter().flatten() {
        if s.trim().is_empty() { return Err(Error("empty filter label".into())); }
    }
    let catalog = match &o.catalog {
        Some(c) => { c.validate()?; c },
        None => BUILTIN.get_or_init(|| Catalog::from_json(include_str!("../data/core.json"))
            .expect("bundled catalog must pass validation")),
    };
    let seed = o.seed.unwrap_or_else(|| rand::thread_rng().gen());
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut warnings = vec![];
    let mut history = vec![];
    for q in &o.recent {
        if let Some(d) = catalog.resolve(q) {
            if !history.iter().any(|h: &&Dish| h.id == d.id) { history.push(d); }
        } else { warnings.push(format!("UNKNOWN_RECENT:{q}")); }
    }
    if !o.exclude_ingredients.is_empty() && !o.require_complete_ingredients {
        warnings.push("INGREDIENT_EXCLUSION_USES_KNOWN_INGREDIENTS_ONLY".into());
    }
    let mut candidates: Vec<&Dish> = catalog.dishes.iter().filter(|d| {
        (o.categories.is_empty() || has(&o.categories, &d.category))
        && (o.regions.is_empty() || intersects(&o.regions, &d.regions))
        && o.meal.as_ref().is_none_or(|v| has(&d.meals, v))
        && o.mode.as_ref().is_none_or(|v| has(&d.modes, v))
        && o.theme.as_ref().is_none_or(|v| has(&d.themes, v))
        && o.max_minutes.is_none_or(|v| d.minutes_max.is_some_and(|m| m <= v))
        && o.budget_max_fen.is_none_or(|v| d.cost_max_fen.is_some_and(|c| c <= v))
        && o.required_tags.iter().all(|v| has(&d.tags, v))
        && !intersects(&o.exclude_ingredients, &d.ingredients)
        && (o.exclude_ingredients.is_empty() || !o.require_complete_ingredients || d.ingredients_complete)
        && (!o.exclude_recent || !history.iter().any(|h| h.id == d.id))
    }).collect();
    candidates.sort_by(|a,b| a.id.cmp(&b.id));
    let candidate_count = candidates.len();
    let mut chosen: Vec<&Dish> = vec![];
    let mut items = vec![];
    while !candidates.is_empty() && items.len() < o.count {
        let weights: Vec<f64> = candidates.iter().map(|d| weight(d, o, &history, &chosen)).collect();
        let index = if o.strategy == Strategy::Uniform {
            weighted_index(&weights, &mut rng)
        } else {
            // Group mass is mean, not sum: adding variants does not inflate a category's mass.
            let mut groups: BTreeMap<String, Vec<usize>> = BTreeMap::new();
            for (i, d) in candidates.iter().enumerate() { groups.entry(norm(&d.category)).or_default().push(i); }
            let groups: Vec<Vec<usize>> = groups.into_values().collect();
            let group_weights: Vec<f64> = groups.iter().map(|g| g.iter().map(|i| weights[*i]).sum::<f64>() / g.len() as f64).collect();
            let group = &groups[weighted_index(&group_weights, &mut rng)];
            group[weighted_index(&group.iter().map(|i| weights[*i]).collect::<Vec<_>>(), &mut rng)]
        };
        let d = candidates.remove(index);
        let mut reasons = vec!["HARD_FILTERS_SATISFIED".into()];
        if o.strategy != Strategy::Uniform && o.preferences.iter().any(|v| has(&d.tags,v) || has(&d.ingredients,v)) {
            reasons.push("MATCH_PREFERENCE".into());
        }
        if !history.is_empty() && !history.iter().any(|h| h.id == d.id) { reasons.push("NOT_IN_RECENT".into()); }
        chosen.push(d);
        items.push(Recommendation { dish: d.clone(), reason_codes: reasons });
    }
    let status = if items.is_empty() { "no_match" } else if items.len() < o.count { "partial" } else { "ok" };
    Ok(Response {
        items, status: status.into(), candidate_count, requested_count: o.count, seed,
        dataset_id: catalog.id.clone(), dataset_version: catalog.version.clone(),
        algorithm_version: "1".into(), warnings,
    })
}
