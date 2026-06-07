use crate::apps::all_apps_identifiers;

pub fn list_apps_ids_command() {
    for id in all_apps_identifiers() {
        println!("{}", id);
    }
}
