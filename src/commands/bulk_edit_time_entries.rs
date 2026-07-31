use crate::api::client::ApiClient;
use crate::error::ArgumentError;
use crate::models::{Entities, ResultWithDefaultError, TimeEntry};
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

        if can_apply_patch_locally(&patch) {
            for entry in selected_entries {
                let updated = apply_patch_to_entry(entry, &patch, &entities)?;
                api_client.update_time_entry(updated).await?;
            }
        } else {
            api_client
                .bulk_update_time_entries(workspace_id, ids.clone(), patch)
                .await?;
        }

        println!(
            "{}",
            format!("Bulk updated {} time entries", ids.len()).green()
        );
        Ok(())
    }
}

fn can_apply_patch_locally(patch: &Value) -> bool {
    let Some(operations) = patch.as_array() else {
        return false;
    };

    operations.iter().all(|operation| {
        matches!(
            (
                operation.get("op").and_then(Value::as_str),
                operation.get("path").and_then(Value::as_str),
            ),
            (
                Some("add" | "replace" | "remove"),
                Some("/description" | "/billable" | "/project_id" | "/pid" | "/tags")
            )
        )
    })
}

fn apply_patch_to_entry(
    mut entry: TimeEntry,
    patch: &Value,
    entities: &Entities,
) -> ResultWithDefaultError<TimeEntry> {
    let operations = patch.as_array().ok_or_else(|| {
        Box::new(ArgumentError::MissingArgument(
            "bulk-edit --json must be a JSON Patch array".to_string(),
        )) as Box<dyn std::error::Error + Send>
    })?;

    for operation in operations {
        let op = operation
            .get("op")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let path = operation
            .get("path")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let value = operation.get("value");

        match (op, path) {
            ("remove", "/description") => entry.description.clear(),
            ("add" | "replace", "/description") => {
                entry.description = value
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
            }
            ("remove", "/billable") => entry.billable = false,
            ("add" | "replace", "/billable") => {
                entry.billable = value.and_then(Value::as_bool).unwrap_or(false);
            }
            ("remove", "/project_id" | "/pid") => {
                entry.project = None;
                entry.task = None;
            }
            ("add" | "replace", "/project_id" | "/pid") => {
                let project_id = value.and_then(Value::as_i64).ok_or_else(|| {
                    Box::new(ArgumentError::MissingArgument(
                        "bulk-edit /project_id value must be a numeric project id".to_string(),
                    )) as Box<dyn std::error::Error + Send>
                })?;
                entry.project =
                    Some(entities.projects.get(&project_id).cloned().ok_or_else(|| {
                        Box::new(ArgumentError::ResourceNotFound(format!(
                            "project id {project_id}"
                        ))) as Box<dyn std::error::Error + Send>
                    })?);
                entry.task = None;
            }
            ("remove", "/tags") => entry.tags.clear(),
            ("add" | "replace", "/tags") => {
                let tags = value.and_then(Value::as_array).ok_or_else(|| {
                    Box::new(ArgumentError::MissingArgument(
                        "bulk-edit /tags value must be an array of tag names".to_string(),
                    )) as Box<dyn std::error::Error + Send>
                })?;
                entry.tags = tags
                    .iter()
                    .filter_map(Value::as_str)
                    .map(ToString::to_string)
                    .collect();
            }
            _ => {}
        }
    }

    Ok(entry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::client::MockApiClient;
    use crate::error::ApiError;
    use crate::models::{Entities, Project, TimeEntry};
    use chrono::{TimeZone, Utc};
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

    fn mock_project(id: i64, workspace_id: i64) -> Project {
        Project {
            id,
            name: format!("Project {id}"),
            workspace_id,
            client: None,
            is_private: false,
            active: true,
            at: Utc.timestamp_opt(1_700_000_000, 0).single().unwrap(),
            created_at: Utc.timestamp_opt(1_700_000_000, 0).single().unwrap(),
            color: "#06aaf5".to_string(),
            billable: None,
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
            let mut entities =
                mock_entities(vec![time_entry_with_id(1, 10), time_entry_with_id(2, 10)]);
            entities.projects.insert(42, mock_project(42, 10));
            Ok(entities)
        });
        api_client
            .expect_update_time_entry()
            .times(2)
            .withf(|entry| {
                entry.description == "focus"
                    && entry.tags == vec!["deep-work"]
                    && entry.project.as_ref().map(|project| project.id) == Some(42)
            })
            .returning(|entry| Ok(entry.id));

        let result = BulkEditTimeEntriesCommand::execute(
            api_client,
            vec![1, 2],
            r#"[{"op":"replace","path":"/description","value":"focus"},{"op":"replace","path":"/project_id","value":42},{"op":"replace","path":"/tags","value":["deep-work"]}]"#.to_string(),
        )
        .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn bulk_edit_uses_bulk_api_for_unknown_patch_paths() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_entities()
            .returning(|| Ok(mock_entities(vec![time_entry_with_id(1, 10)])));
        api_client
            .expect_bulk_update_time_entries()
            .withf(|workspace_id, ids, patch| {
                *workspace_id == 10
                    && ids == &vec![1]
                    && patch == &json!([{ "op": "replace", "path": "/unknown", "value": "x" }])
            })
            .returning(|_, _, _| Ok(json!({})));

        let result = BulkEditTimeEntriesCommand::execute(
            api_client,
            vec![1],
            r#"[{"op":"replace","path":"/unknown","value":"x"}]"#.to_string(),
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
            .expect_update_time_entry()
            .returning(|_| Err(Box::new(ApiError::Network)));

        let result = BulkEditTimeEntriesCommand::execute(
            api_client,
            vec![1],
            r#"[{"op":"replace","path":"/description","value":"focus"}]"#.to_string(),
        )
        .await;
        assert_err!(result);
    }
}
