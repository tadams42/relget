use std::collections::HashMap;
use std::sync::OnceLock;

use rust_embed::RustEmbed;
use serde::Deserialize;

#[derive(RustEmbed)]
#[folder = "src/apps/"]
#[include = "registry.yaml"]
struct RegistryAsset;

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ManPagesStatus {
    Unavailable,
    Bundled,
    SelfGenerated,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ShellCompletionsStatus {
    Unavailable,
    Bundled,
    SelfGenerated,
}

#[derive(Debug)]
pub struct AppEntry {
    pub id:                String,
    pub exe_name:          String,
    pub url:               String,
    pub category:          String,
    pub description:       String,
    pub has_musl:          bool,
    pub man_pages:         ManPagesStatus,
    pub shell_completions: ShellCompletionsStatus,
}

#[derive(Debug)]
pub struct CategoryInfo {
    pub key:         String,
    pub title:       String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
struct RawAppEntry {
    id:                String,
    exe_name:          String,
    url:               String,
    description:       String,
    has_musl:          bool,
    man_pages:         ManPagesStatus,
    shell_completions: ShellCompletionsStatus,
}

#[derive(Deserialize)]
struct RawCategory {
    title:       Option<String>,
    description: Option<String>,
    apps:        Vec<RawAppEntry>,
}

struct Registry {
    entries:    Vec<AppEntry>,
    categories: Vec<CategoryInfo>,
}

static REGISTRY: OnceLock<Registry> = OnceLock::new();

fn registry() -> &'static Registry {
    REGISTRY.get_or_init(|| {
        let file = RegistryAsset::get("registry.yaml").expect("registry.yaml embedded");
        let raw: HashMap<String, RawCategory> =
            serde_yaml::from_slice(&file.data).expect("valid registry.yaml");

        let mut cat_keys: Vec<String> = raw.keys().cloned().collect();
        cat_keys.sort();

        let mut entries = Vec::new();
        let mut categories = Vec::new();

        for key in cat_keys {
            let cat = raw.get(&key).expect("key present");
            categories.push(CategoryInfo {
                key:         key.clone(),
                title:       cat.title.clone().unwrap_or_else(|| key.clone()),
                description: cat.description.clone(),
            });
            for app in &cat.apps {
                entries.push(AppEntry {
                    id:                app.id.clone(),
                    exe_name:          app.exe_name.clone(),
                    url:               app.url.clone(),
                    category:          key.clone(),
                    description:       app.description.clone(),
                    has_musl:          app.has_musl,
                    man_pages:         app.man_pages.clone(),
                    shell_completions: app.shell_completions.clone(),
                });
            }
        }

        Registry { entries, categories }
    })
}

pub fn all_app_entries() -> &'static [AppEntry] { &registry().entries }

pub fn all_categories() -> &'static [CategoryInfo] { &registry().categories }

pub fn all_apps_identifiers() -> Vec<&'static str> {
    let mut ids: Vec<&str> = registry().entries.iter().map(|e| e.id.as_str()).collect();
    ids.sort_unstable();
    ids
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::apps::create_app;
    use std::collections::HashSet;

    #[test]
    fn registry_exe_names_match_app_trait() {
        for entry in all_app_entries() {
            let app = create_app(&entry.id, None, None, None, true)
                .unwrap_or_else(|| panic!("create_app returned None for id '{}'", entry.id));
            assert_eq!(
                entry.exe_name,
                app.exe_name(),
                "registry exe_name mismatch for id '{}': yaml='{}' trait='{}'",
                entry.id,
                entry.exe_name,
                app.exe_name()
            );
        }
    }

    #[test]
    fn registry_ids_are_unique() {
        let ids: Vec<_> = all_app_entries().iter().map(|e| e.id.as_str()).collect();
        let unique: HashSet<_> = ids.iter().copied().collect();
        assert_eq!(ids.len(), unique.len(), "duplicate app ids found in registry.yaml");
    }

    #[test]
    fn all_apps_identifiers_is_sorted_and_matches_entries() {
        let identifiers = all_apps_identifiers();
        let is_sorted = identifiers.windows(2).all(|w| w[0] <= w[1]);
        assert!(is_sorted, "all_apps_identifiers() is not sorted");

        let entry_ids: HashSet<_> = all_app_entries().iter().map(|e| e.id.as_str()).collect();
        let identifier_set: HashSet<_> = identifiers.iter().copied().collect();
        assert_eq!(
            entry_ids, identifier_set,
            "all_apps_identifiers() does not match the set of ids from all_app_entries()"
        );
    }

    #[test]
    fn all_apps_have_factory_entry() {
        for entry in all_app_entries() {
            assert!(
                create_app(&entry.id, None, None, None, true).is_some(),
                "create_app returned None for registry id '{}'",
                entry.id
            );
        }
    }

    #[test]
    fn all_apps_have_binary_descriptor() {
        // assets() is the source of truth for the uninstaller — it must describe every
        // installed file. The primary binary descriptor being present is a minimal check.
        // The full assets() <-> download() invariant requires an integration test (needs network).
        for entry in all_app_entries() {
            let app = create_app(&entry.id, None, None, None, true)
                .unwrap_or_else(|| panic!("create_app returned None for id '{}'", entry.id));
            assert!(
                app.assets().binary.is_some(),
                "app '{}' has no primary binary descriptor in assets()",
                entry.id
            );
        }
    }
}
