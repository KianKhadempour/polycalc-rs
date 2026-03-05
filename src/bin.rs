use combat_core::data::loader::Loader;

fn main() {
    let loader = Loader::new("unit_data.toml").unwrap();

    let unit_registry = loader.load();

    dbg!(&unit_registry.definitions[0]);
}
