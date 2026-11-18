//! `spec/frame.schema.json` is generated from the Rust types. Run with
//! `ULANA_BLESS=1` to write it after changing a block.

use std::path::PathBuf;

#[test]
fn the_schema_file_matches_the_types() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../spec/frame.schema.json");
    let schema = serde_json::to_string_pretty(&ulana::frame_schema()).unwrap() + "\n";
    if std::env::var_os("ULANA_BLESS").is_some() {
        std::fs::write(&path, &schema).unwrap();
        return;
    }
    let saved = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(
        saved == schema,
        "spec/frame.schema.json is out of date; run `ULANA_BLESS=1 cargo test -p ulana --test schema`"
    );
}
