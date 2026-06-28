use std::sync::Arc;

use super::App;
use super::coding::RustAnalyzer;
use super::generic_app::GenericApp;
use crate::{GithubClient, Registry, RelgetClient};

pub fn create_app(
    id: &str, gh_token: Option<String>, cb_token: Option<String>, gl_token: Option<String>,
    offline: bool,
) -> Option<Box<dyn App>> {
    match id {
        RustAnalyzer::ID => {
            let client: Arc<dyn RelgetClient> = Arc::new(GithubClient::new(gh_token, offline));
            Some(Box::new(RustAnalyzer::new(client)))
        }
        _ => {
            let entry = Registry::global()
                .entries()
                .iter()
                .find(|e| e.id == id)?
                .clone();
            let client = GenericApp::client_for(&entry, gh_token, cb_token, gl_token, offline);
            Some(Box::new(GenericApp::new(entry, client)))
        }
    }
}
