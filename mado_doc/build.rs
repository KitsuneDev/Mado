use mado::events::Event;
use schemars::{Schema, schema_for};
use serde_json::Value;
use std::{env, fs, path::PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let src_dir = manifest_dir.join("src");
    let docs_dir = manifest_dir.join("docs");
    let _ = fs::remove_dir_all(&docs_dir);
    let commands_docs = &docs_dir.join("commands");
    let events_docs = &docs_dir.join("events");
    let _ = fs::create_dir_all(&commands_docs);
    let _ = fs::create_dir_all(&events_docs);
    wry_cmd::generate_docs(&[src_dir], &commands_docs).expect("failed to generate command docs");

    let event_schema = schema_for!(Event);
    let event_json = serde_json::to_string_pretty(&event_schema).unwrap();
    fs::write(events_docs.join("schema.json"), event_json).unwrap();
}
