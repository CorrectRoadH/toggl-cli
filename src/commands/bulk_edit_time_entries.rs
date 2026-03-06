use crate::api::client::ApiClient;
use crate::error::ArgumentError;
use crate::models::ResultWithDefaultError;
use colored::Colorize;
use serde_json::Value;

pub struct BulkEditTimeEntriesCommand;

impl BulkEditTimeEntriesCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        ids: Vec<i64>,
        payload: String,
    ) -> ResultWithDefaultError<()> {
        let patch: Value = serde_json::from_str(&payload)
            .map_err(|error| -> Box<dyn std::error::Error + Send> { Box::new(error) })?;

        let entities = api_client.get_entities().await?;
        let selected_entries = ids
            .iter()
            .filter_map(|id| {
                entities
                    .time_entries
                    .iter()
                    .find(|time_entry| time_entry.id == *id)
                    .cloned()
            })
            .collect::<Vec<_>>();

        if selected_entries.len() != ids.len() {
            let found_ids = selected_entries
                .iter()
                .map(|entry| entry.id)
                .collect::<std::collections::HashSet<_>>();
            let missing_ids = ids
                .iter()
                .copied()
                .filter(|id| !found_ids.contains(id))
                .collect::<Vec<_>>();
            println!(
                "{}",
                format!("No time entries found with ids {:?}", missing_ids).yellow()
            );
            return Ok(());
        }

        let workspace_id = selected_entries
            .first()
            .map(|entry| entry.workspace_id)
            .unwrap_or_default();

        if selected_entries
            .iter()
            .any(|entry| entry.workspace_id != workspace_id)
        {
            return Err(Box::new(ArgumentError::MultipleWorkspaces(
                "Bulk edit only supports time entries from the same workspace".to_string(),
            )));
        }

        api_client
            .bulk_update_time_entries(workspace_id, ids.clone(), patch)
            .await?;

        println!(
            "{}",
            format!("Bulk updated {} time entries", ids.len()).green()
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::client::MockApiClient;
    use crate::error::ApiError;
    use crate::models::{Entities, TimeEntry};
    use chrono::Utc;
    use serde_json::json;
    use std::collections::HashMap;
    use tokio_test::{assert_err, assert_ok};

    fn time_entry_with_id(id: i64, workspace_id: i64) -> TimeEntry {
        TimeEntry {
            id,
            workspace_id,
            start: Utc::now(),
            duration: 60,
            ..Default::default()
        }
    }

    fn mock_entities(entries: Vec<TimeEntry>) -> Entities {
        Entities {
            time_entries: entries,
            projects: HashMap::new(),
            tasks: HashMap::new(),
            clients: HashMap::new(),
            workspaces: Vec::new(),
            tags: Vec::new(),
        }
    }

    #[tokio::test]
    async fn bulk_edit_returns_ok_on_success() {
        let mut api_client = MockApiClient::new();
        api_client.expect_get_entities().returning(|| {
            Ok(mock_entities(vec![
                time_entry_with_id(1, 10),
                time_entry_with_id(2, 10),
            ]))
        });
        api_client
            .expect_bulk_update_time_entries()
            .withf(|workspace_id, ids, patch| {
                *workspace_id == 10
                    && ids == &vec![1, 2]
                    && patch
                        == &json!([{ "op": "replace", "path": "/description", "value": "focus" }])
            })
            .returning(|_, _, _| Ok(json!({})));

        let result = BulkEditTimeEntriesCommand::execute(
            api_client,
            vec![1, 2],
            r#"[{"op":"replace","path":"/description","value":"focus"}]"#.to_string(),
        )
        .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn bulk_edit_handles_missing_ids() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_entities()
            .returning(|| Ok(mock_entities(vec![time_entry_with_id(1, 10)])));

        let result = BulkEditTimeEntriesCommand::execute(
            api_client,
            vec![1, 2],
            r#"[{"op":"replace","path":"/description","value":"focus"}]"#.to_string(),
        )
        .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn bulk_edit_returns_error_for_multiple_workspaces() {
        let mut api_client = MockApiClient::new();
        api_client.expect_get_entities().returning(|| {
            Ok(mock_entities(vec![
                time_entry_with_id(1, 10),
                time_entry_with_id(2, 20),
            ]))
        });

        let result = BulkEditTimeEntriesCommand::execute(
            api_client,
            vec![1, 2],
            r#"[{"op":"replace","path":"/description","value":"focus"}]"#.to_string(),
        )
        .await;
        assert_err!(result);
    }

    #[tokio::test]
    async fn bulk_edit_returns_error_on_invalid_json() {
        let api_client = MockApiClient::new();
        let result =
            BulkEditTimeEntriesCommand::execute(api_client, vec![1], "not-json".to_string()).await;
        assert_err!(result);
    }

    #[tokio::test]
    async fn bulk_edit_returns_error_on_api_failure() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_entities()
            .returning(|| Ok(mock_entities(vec![time_entry_with_id(1, 10)])));
        api_client
            .expect_bulk_update_time_entries()
            .returning(|_, _, _| Err(Box::new(ApiError::Network)));

        let result = BulkEditTimeEntriesCommand::execute(
            api_client,
            vec![1],
            r#"[{"op":"replace","path":"/description","value":"focus"}]"#.to_string(),
        )
        .await;
        assert_err!(result);
    }
}
