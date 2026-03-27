use crate::api::client::ApiClient;
use crate::error::ArgumentError;
use crate::models::{ResultWithDefaultError, TimeEntry};
use std::io::{BufWriter, Write};

/// Common utilities for command implementations
pub struct CommandUtils;

impl CommandUtils {
    /// Get workspace ID from API client
    pub async fn get_workspace_id(api_client: &impl ApiClient) -> ResultWithDefaultError<i64> {
        Ok(api_client.get_user().await?.default_workspace_id)
    }

    /// Find resource by name in a collection
    pub fn find_resource_by_name<T, F>(
        resources: Vec<T>,
        name: &str,
        resource_type: &str,
        name_getter: F,
    ) -> ResultWithDefaultError<T>
    where
        F: Fn(&T) -> &str,
    {
        resources
            .into_iter()
            .find(|resource| name_getter(resource) == name)
            .ok_or_else(|| {
                Box::new(ArgumentError::ResourceNotFound(format!(
                    "No {} found with name '{}'",
                    resource_type, name
                ))) as Box<dyn std::error::Error + Send>
            })
    }

    /// Print success message for resource creation
    pub fn print_creation_success(resource_type: &str, resource_display: &dyn std::fmt::Display) {
        println!(
            "{} created successfully\n{}",
            resource_type, resource_display
        );
    }

    /// Serialize a TimeEntry as pretty-printed JSON to stdout.
    ///
    /// Uses struct field order (via serde_json `preserve_order` feature) and
    /// appends a `"running"` boolean computed from `entry.is_running()`.
    pub fn print_time_entry_json(entry: &TimeEntry) {
        let mut value =
            serde_json::to_value(entry).expect("failed to serialize time entry to JSON");
        if let Some(obj) = value.as_object_mut() {
            obj.insert(
                "running".to_string(),
                serde_json::Value::Bool(entry.is_running()),
            );
        }
        let json_string =
            serde_json::to_string_pretty(&value).expect("failed to serialize to JSON");
        let stdout = std::io::stdout();
        let mut handle = BufWriter::new(stdout);
        writeln!(handle, "{json_string}").expect("failed to print");
    }

    /// Print success message for resource deletion
    pub fn print_deletion_success(resource_type: &str) {
        println!("{} deleted successfully", resource_type);
    }

    /// Print success message for resource update
    #[allow(dead_code)]
    pub fn print_update_success(resource_type: &str, resource_display: &dyn std::fmt::Display) {
        println!(
            "{} updated successfully\n{}",
            resource_type, resource_display
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Client;

    #[test]
    fn find_resource_by_name_returns_matching_resource() {
        let clients = vec![
            Client {
                id: 1,
                name: "Acme".to_string(),
                workspace_id: 1,
            },
            Client {
                id: 2,
                name: "Globex".to_string(),
                workspace_id: 1,
            },
        ];

        let result =
            CommandUtils::find_resource_by_name(clients, "Acme", "client", |client| &client.name);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, 1);
    }

    #[test]
    fn find_resource_by_name_returns_error_when_not_found() {
        let clients = vec![Client {
            id: 1,
            name: "Acme".to_string(),
            workspace_id: 1,
        }];

        let result =
            CommandUtils::find_resource_by_name(clients, "NonExistent", "client", |client| {
                &client.name
            });

        assert!(result.is_err());
    }
}
