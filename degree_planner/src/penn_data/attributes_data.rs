use std::collections::HashMap;
use std::sync::OnceLock;

const ATTRIBUTES_JSON: &str = include_str!("attributes.json");

static ATTRIBUTES: OnceLock<HashMap<String, Vec<String>>> = OnceLock::new();

fn init_attributes() -> &'static HashMap<String, Vec<String>> {
    ATTRIBUTES.get_or_init(|| {
        serde_json::from_str(ATTRIBUTES_JSON)
            .expect("attributes.json must deserialize to HashMap<String, Vec<String>>")
    })
}

/// Borrow the static course-attribute index, parsed once on first access.
pub fn attributes() -> &'static HashMap<String, Vec<String>> {
    init_attributes()
}

/// Clone of the attribute map for callers that need an owned or mutable copy.
pub fn create_attributes() -> HashMap<String, Vec<String>> {
    attributes().clone()
}
