use crate::api::client::ApiClient;
use crate::error::ApiError;
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

        let mut updated_count = 0usize;
        let mut failures: Vec<String> = Vec::new();
        // The API caps a bulk patch at 100 ids per request.
        for chunk in ids.chunks(MAX_IDS_PER_REQUEST) {
            let response = api_client
                .bulk_update_time_entries(workspace_id, chunk.to_vec(), patch.clone())
                .await?;
            let (succeeded, chunk_failures) = split_bulk_response(&response, chunk);
            updated_count += succeeded;
            failures.extend(chunk_failures);
        }

        // The server reports per-entry outcomes, so a patch that only partly
        // landed must not print an unqualified success — that is exactly how
        // the "reported success but nothing changed" bug stayed invisible.
        if !failures.is_empty() {
            for failure in &failures {
                eprintln!("{}", failure.red());
            }
            return Err(Box::new(ApiError::BulkEditPartiallyFailed {
                updated: updated_count,
                failed: failures.len(),
            }));
        }

        println!(
            "{}",
            format!("Bulk updated {} time entries", updated_count).green()
        );
        Ok(())
    }
}

const MAX_IDS_PER_REQUEST: usize = 100;

/// Reads the `{"success": [...], "failure": [{"id", "message"}]}` payload the
/// bulk endpoint returns. A server that answers with neither list (or with a
/// bare object) is treated as having applied the whole chunk, which keeps the
/// command working against older backends.
fn split_bulk_response(response: &Value, requested: &[i64]) -> (usize, Vec<String>) {
    let success = response.get("success").and_then(Value::as_array);
    let failure = response.get("failure").and_then(Value::as_array);
    if success.is_none() && failure.is_none() {
        return (requested.len(), Vec::new());
    }

    let succeeded = success.map(|ids| ids.len()).unwrap_or(0);
    let failures = failure
        .map(|entries| {
            entries
                .iter()
                .map(|entry| {
                    let id = entry
                        .get("id")
                        .and_then(Value::as_i64)
                        .map(|id| id.to_string())
                        .unwrap_or_else(|| "unknown".to_string());
                    let message = entry
                        .get("message")
                        .and_then(Value::as_str)
                        .unwrap_or("no reason given");
                    format!("Time entry {id} was not updated: {message}")
                })
                .collect()
        })
        .unwrap_or_default();

    (succeeded, failures)
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
                        == &json!([
                            { "op": "replace", "path": "/description", "value": "focus" },
                            { "op": "replace", "path": "/project_id", "value": 42 },
                        ])
            })
            .returning(|_, ids, _| Ok(json!({ "success": ids, "failure": [] })));

        let result = BulkEditTimeEntriesCommand::execute(
            api_client,
            vec![1, 2],
            r#"[{"op":"replace","path":"/description","value":"focus"},{"op":"replace","path":"/project_id","value":42}]"#.to_string(),
        )
        .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn bulk_edit_reports_per_entry_failures() {
        let mut api_client = MockApiClient::new();
        api_client.expect_get_entities().returning(|| {
            Ok(mock_entities(vec![
                time_entry_with_id(1, 10),
                time_entry_with_id(2, 10),
            ]))
        });
        api_client
            .expect_bulk_update_time_entries()
            .returning(|_, _, _| {
                Ok(json!({
                    "success": [1],
                    "failure": [{ "id": 2, "message": "time entry not found" }],
                }))
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
    async fn bulk_edit_treats_a_response_without_lists_as_full_success() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_entities()
            .returning(|| Ok(mock_entities(vec![time_entry_with_id(1, 10)])));
        api_client
            .expect_bulk_update_time_entries()
            .returning(|_, _, _| Ok(json!({})));

        let result = BulkEditTimeEntriesCommand::execute(
            api_client,
            vec![1],
            r#"[{"op":"replace","path":"/description","value":"focus"}]"#.to_string(),
        )
        .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn bulk_edit_splits_ids_into_chunks_of_one_hundred() {
        let mut api_client = MockApiClient::new();
        api_client.expect_get_entities().returning(|| {
            Ok(mock_entities(
                (1..=150).map(|id| time_entry_with_id(id, 10)).collect(),
            ))
        });
        api_client
            .expect_bulk_update_time_entries()
            .times(2)
            .withf(|_, ids, _| ids.len() == 100 || ids.len() == 50)
            .returning(|_, ids, _| Ok(json!({ "success": ids, "failure": [] })));

        let result = BulkEditTimeEntriesCommand::execute(
            api_client,
            (1..=150).collect(),
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
