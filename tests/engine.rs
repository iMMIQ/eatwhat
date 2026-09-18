use eatwhat::*;
use std::collections::BTreeSet;

fn options() -> Options { Options { seed: Some(42), ..Default::default() } }
fn catalog() -> Catalog { Catalog::from_json(include_str!("../data/core.json")).unwrap() }

#[test]
fn bundled_catalog_valid_and_unique() {
    let c = catalog();
    assert!(c.dishes.len() >= 70);
    assert_eq!(c.dishes.iter().map(|d| &d.id).collect::<BTreeSet<_>>().len(), c.dishes.len());
}
#[test]
fn deterministic_without_replacement() {
    let o = Options { count: 30, ..options() };
    let a = recommend(&o).unwrap();
    assert_eq!(a, recommend(&o).unwrap());
    assert_eq!(a.items.len(), 30);
    assert_eq!(a.items.iter().map(|i| &i.dish.id).collect::<BTreeSet<_>>().len(), 30);
}
#[test]
fn hard_filters_are_never_relaxed() {
    let o = Options { count: 100, mode: Some("cook".into()), max_minutes: Some(25),
        budget_max_fen: Some(1200), categories: vec!["米饭".into()], ..options() };
    let r = recommend(&o).unwrap();
    assert_eq!(r.status, "partial");
    assert!(!r.items.is_empty());
    for i in r.items { assert_eq!(i.dish.category, "米饭"); assert!(i.dish.minutes_max.unwrap() <= 25); assert!(i.dish.cost_max_fen.unwrap() <= 1200); }
    let r = recommend(&Options { required_tags: vec!["不存在的口味".into()], ..options() }).unwrap();
    assert_eq!(r.status, "no_match");
    assert_eq!(r.candidate_count, 0);
}
#[test]
fn aliases_exclude_recent() {
    let o = Options { count: 100, recent: vec!["西红柿炒鸡蛋".into()], ..options() };
    assert!(recommend(&o).unwrap().items.iter().all(|i| i.dish.name != "番茄炒蛋"));
}
#[test]
fn missing_ingredient_data_fails_closed() {
    let o = Options { exclude_ingredients: vec!["花生".into()], ..options() };
    assert_eq!(recommend(&o).unwrap().status, "no_match");
    let r = recommend(&Options { count: 100, require_complete_ingredients: false, ..o }).unwrap();
    assert!(!r.items.is_empty());
    assert!(!r.warnings.is_empty());
    assert!(r.items.iter().all(|i| !i.dish.ingredients.contains(&"花生".to_string())));
}
#[test]
fn custom_catalog_and_unknown_numeric_fields() {
    let mut c = catalog(); c.dishes.truncate(1); c.dishes[0].minutes_max = None;
    let o = Options { catalog: Some(c), max_minutes: Some(1000), ..options() };
    assert_eq!(recommend(&o).unwrap().status, "no_match");
}
#[test]
fn duplicate_alias_and_invalid_sources_rejected() {
    let mut c = catalog(); let first_name = c.dishes[0].name.clone(); c.dishes[1].aliases.push(first_name);
    assert!(c.validate().is_err());
    let mut c = catalog(); c.dishes[0].sources = vec!["missing".into()];
    assert!(c.validate().is_err());
}
#[test]
fn invalid_options_and_unknown_keys_rejected() {
    assert!(recommend(&Options { count: 0, ..options() }).is_err());
    assert!(recommend(&Options { count: 101, ..options() }).is_err());
    assert!(recommend(&Options { budget_max_fen: Some(1000), ..options() }).is_err());
    assert!(serde_json::from_str::<Options>(r#"{"sead":42}"#).is_err());
    assert!(serde_json::from_str::<Options>(r#"{"strategy":"oops"}"#).is_err());
}
#[test]
fn input_order_does_not_change_seeded_output() {
    let c = catalog(); let mut reversed = c.clone(); reversed.dishes.reverse();
    let a = recommend(&Options { catalog: Some(c), count: 10, ..options() }).unwrap();
    let b = recommend(&Options { catalog: Some(reversed), count: 10, ..options() }).unwrap();
    assert_eq!(a, b);
}
#[test]
fn grouped_sampling_resists_category_size_bias() {
    let mut c = catalog(); c.dishes.truncate(10);
    for (i,d) in c.dishes.iter_mut().enumerate() {
        d.category = if i == 0 { "small" } else { "large" }.into();
        d.familiarity = 0.5;
    }
    let mut balanced_small = 0; let mut uniform_small = 0;
    for seed in 0..2000 {
        for (strategy, total) in [(Strategy::Balanced, &mut balanced_small), (Strategy::Uniform, &mut uniform_small)] {
            let r = recommend(&Options { catalog: Some(c.clone()), seed: Some(seed), strategy, ..Default::default() }).unwrap();
            if r.items[0].dish.category == "small" { *total += 1; }
        }
    }
    assert!((800..1200).contains(&balanced_small), "{balanced_small}");
    assert!((100..300).contains(&uniform_small), "{uniform_small}");
}
#[test]
fn no_history_state_leaks_across_calls() {
    let a = recommend(&options()).unwrap();
    let _ = recommend(&Options { recent: vec![a.items[0].dish.id.clone()], ..options() });
    assert_eq!(a, recommend(&options()).unwrap());
}
#[test]
fn all_strategies_respect_constraints() {
    for strategy in [Strategy::Uniform, Strategy::Balanced, Strategy::Comfort, Strategy::Explore, Strategy::Surprise] {
        let r = recommend(&Options { strategy, count: 100, theme: Some("quick_meal".into()), ..options() }).unwrap();
        assert!(!r.items.is_empty());
        assert!(r.items.iter().all(|i| i.dish.themes.contains(&"quick_meal".to_string())));
    }
}
