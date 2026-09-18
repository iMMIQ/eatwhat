use eatwhat::{recommend, Options};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = recommend(&Options {
        seed: Some(42), count: 3, max_minutes: Some(30),
        mode: Some("cook".into()), preferences: vec!["酸鲜".into()],
        ..Default::default()
    })?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
