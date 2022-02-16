use super::issue::{PdIssue, PdIssueFields};
use anyhow::Result;
use futures::future::try_join_all;
use log::*;

pub struct PdReporter {
    pagerduty_api_key: Option<String>,
    to_trigger: Vec<PdIssue>,
    to_resolve: Vec<PdIssue>,
}

impl PdReporter {
    pub fn new(pagerduty_api_key: Option<String>) -> Result<Self> {
        Ok(Self {
            pagerduty_api_key,
            to_trigger: Default::default(),
            to_resolve: Default::default(),
        })
    }

    pub fn trigger(&mut self, issue: PdIssue) {
        self.to_trigger.push(issue);
    }

    pub fn resolve(&mut self, issue: PdIssue) {
        self.to_resolve.push(issue);
    }

    pub async fn finish(self) -> Result<()> {
        let pd_client = match self.pagerduty_api_key {
            None => {
                warn!("🌵 DRY RUN, nothing was sent to PagerDuty");
                return Ok(());
            }
            Some(key) => pagerduty_rs::eventsv2async::EventsV2::new(key, None)?,
        };
        let mut tasks = Vec::new();

        for issue in self.to_trigger.into_iter() {
            warn!("🚨 Triggering issue {}: {:?}", issue.dedup_key(), issue);
            let event = pagerduty_rs::types::Event::AlertTrigger(issue.into());
            tasks.push(pd_client.event(event));
        }

        for issue in self.to_resolve.into_iter() {
            warn!("✅ Resolving issue {}: {:?}", issue.dedup_key(), issue);
            let event = pagerduty_rs::types::Event::<PdIssueFields>::AlertResolve(issue.into());
            tasks.push(pd_client.event(event));
        }

        debug!("Waiting for {} Pagerduty update tasks", tasks.len());
        try_join_all(tasks).await?;
        debug!("Finished sending updates to PagerDuty");

        Ok(())
    }
}
