use std::path::Path;

use ra_ap_hir::{db::HirDatabase, Adt, Crate, HasCrate, Struct, Trait};
use ra_ap_project_model::{CargoConfig, InvocationStrategy, RustcSource, UnsetTestCrates};
use ra_ap_rust_analyzer::cli::load_cargo::{load_workspace_at, LoadCargoConfig};

#[derive(Debug)]
pub struct PluginCrate {
    pub name: String,
    pub components: Vec<PluginComponent>,
}

#[derive(Debug)]
pub struct PluginComponent {
    pub name: String,
    pub path: String,
    pub fields: Vec<String>,
}

fn find_crate(name: &str, db: &dyn HirDatabase) -> Option<Crate> {
    let crates = Crate::all(db);
    crates
        .iter()
        .find(|krate| {
            if let Some(display_name) = krate.display_name(db) {
                display_name.canonical_name() == name
            } else {
                false
            }
        })
        .cloned()
}

fn find_trait(name: &str, krate: &Crate, db: &dyn HirDatabase) -> Option<Trait> {
    krate
        .modules(db)
        .iter()
        .flat_map(|module| module.declarations(db))
        .find_map(|declaration| {
            let ra_ap_hir::ModuleDef::Trait(trait_) = declaration else {
                return None
            };
            let Some(trait_name) = trait_.name(db).as_text() else {
                return None
            };
            if trait_name == name {
                return Some(trait_);
            }
            None
        })
}

fn build_struct_path(struct_: Struct, db: &dyn HirDatabase) -> String {
    let mut working_name = struct_.name(db).as_text().unwrap().to_string();
    let mut maybe_module = Some(struct_.module(db));
    while let Some(module) = maybe_module {
        if let Some(module_name) = module.name(db).and_then(|n| n.as_text()) {
            working_name = format!("{}::{}", module_name, working_name);
        };
        maybe_module = module.parent(db);
    }
    let crate_display_name = struct_.krate(db).display_name(db).unwrap();
    let crate_name = crate_display_name.canonical_name();
    format!("{}::{}", crate_name, working_name)
}

pub fn analyze(name: &str, path: &Path) -> PluginCrate {
    let cargo_config = CargoConfig {
        sysroot: Some(RustcSource::Discover),
        invocation_strategy: InvocationStrategy::Once,
        unset_test_crates: UnsetTestCrates::All,
        ..Default::default()
    };

    let load_cargo_config = LoadCargoConfig {
        load_out_dirs_from_check: true,
        with_proc_macro: true,
        prefill_caches: false,
    };

    let (host, _, _) = load_workspace_at(path, &cargo_config, &load_cargo_config, &|_| {}).unwrap();

    let db = host.raw_database();
    let bevy_ecs = find_crate("bevy_ecs", db).expect("Did not find bevy_ecs");
    let component = find_trait("Component", &bevy_ecs, db).expect("Did not find Component");

    let plugin_crate = find_crate(name, db).expect("Did not find plugin crate");

    let components = plugin_crate
        .modules(db)
        .iter()
        .flat_map(|module| module.impl_defs(db))
        .filter(|impl_def| impl_def.trait_(db) == Some(component))
        .filter_map(|impl_def| {
            let Some(Adt::Struct(impl_struct)) =  impl_def.self_ty(db).as_adt() else {
            return None;
        };

            Some(PluginComponent {
                name: impl_struct.name(db).to_string(),
                path: build_struct_path(impl_struct, db),
                fields: impl_struct
                    .fields(db)
                    .iter()
                    .map(|field| field.name(db).to_string())
                    .collect(),
            })
        })
        .collect();
    PluginCrate {
        name: name.to_string(),
        components,
    }
}
