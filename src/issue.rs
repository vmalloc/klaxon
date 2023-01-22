use serde::Serialize;
use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

const MAX_SUMMARY_SIZE: usize = 1024;

pub type PdIssueFields = BTreeMap<String, String>;

#[derive(Debug, Serialize)]
pub struct PdIssue {
    pub title: String,
    pub source: String,
    pub component: String,
    pub dedup_fields: PdIssueFields,
}

impl PdIssue {
    pub fn dedup_key(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

impl Hash for PdIssue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Not all fields are hashed so the issue can be updated while using the same dedup key
        std::any::type_name::<Self>().hash(state);
        for (key, value) in &self.dedup_fields {
            key.hash(state);
            value.hash(state);
        }
    }
}

impl From<PdIssue> for pagerduty_rs::types::AlertTrigger<PdIssueFields> {
    fn from(src: PdIssue) -> Self {
        Self {
            dedup_key: Some(src.dedup_key().to_string()),
            payload: pagerduty_rs::types::AlertTriggerPayload {
                severity: pagerduty_rs::types::Severity::Critical,
                summary: if src.title.len() > MAX_SUMMARY_SIZE {
                    let mut new_title = src.title;
                    new_title.truncate(MAX_SUMMARY_SIZE - 3);
                    new_title.push_str("...");
                    new_title
                } else {
                    src.title
                },
                source: src.source,
                component: Some(src.component),
                custom_details: Some(src.dedup_fields),
                class: None,
                group: None,
                timestamp: None,
            },
            client: None,
            client_url: None,
            images: None,
            links: None,
        }
    }
}

impl From<PdIssue> for pagerduty_rs::types::AlertResolve {
    fn from(src: PdIssue) -> Self {
        Self {
            dedup_key: src.dedup_key().to_string(),
        }
    }
}
