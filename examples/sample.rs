use bevy_plugin_analyzer::analyze;

fn main() {
    let path = std::fs::canonicalize(std::path::PathBuf::from("./examples/sample_plugin")).unwrap();
    let plugin = analyze("sample_plugin", path.as_path());

    println!("{:#?}", plugin);
}
