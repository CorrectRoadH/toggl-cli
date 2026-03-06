use crate::api;
use crate::arguments::Entity;
use crate::models;
use api::client::ApiClient;
use colored::Colorize;
use models::ResultWithDefaultError;
use std::io::{self, BufWriter, Write};

pub struct ListCommand;

impl ListCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        count: Option<usize>,
        json_flag: bool,
        since: Option<String>,
        until: Option<String>,
        entity: Option<Entity>,
    ) -> ResultWithDefaultError<()> {
        let is_time_entry = matches!(entity, None | Some(Entity::TimeEntry { .. }));
        let has_date_filter = since.is_some() || until.is_some();

        if is_time_entry && has_date_filter {
            let stdout = io::stdout();
            let mut handle = BufWriter::new(stdout);
            let json = match &entity {
                Some(Entity::TimeEntry { json }) => json_flag || *json,
                _ => json_flag,
            };
            match api_client.get_time_entries_filtered(since, until).await {
                Err(error) => println!(
                    "{}\n{}",
                    "Couldn't fetch time entries from API".red(),
                    error
                ),
                Ok(entries) => {
                    let entries = entries
                        .iter()
                        .take(count.unwrap_or(usize::MAX))
                        .collect::<Vec<_>>();
                    if json {
                        let json_string = serde_json::to_string_pretty(&entries)
                            .expect("failed to serialize time entries to JSON");
                        writeln!(handle, "{json_string}").expect("failed to print");
                    } else {
                        entries
                            .iter()
                            .for_each(|te| writeln!(handle, "{te}").expect("failed to print"));
                    }
                }
            }
            return Ok(());
        }

        if let Some(Entity::Tag { json: entity_json }) = entity {
            let json = json_flag || entity_json;
            let user = api_client.get_user().await?;
            match api_client.get_tags(user.default_workspace_id).await {
                Err(error) => println!("{}\n{}", "Couldn't fetch tags from API".red(), error),
                Ok(tags) => {
                    let stdout = io::stdout();
                    let mut handle = BufWriter::new(stdout);
                    let tags = tags
                        .iter()
                        .take(count.unwrap_or(usize::MAX))
                        .collect::<Vec<_>>();
                    if json {
                        let json_string = serde_json::to_string_pretty(&tags)
                            .expect("failed to serialize tags to JSON");
                        writeln!(handle, "{json_string}").expect("failed to print");
                    } else {
                        tags.iter()
                            .for_each(|tag| writeln!(handle, "{tag}").expect("failed to print"));
                    }
                }
            }
            return Ok(());
        }

        if let Some(Entity::Client { json: entity_json }) = entity {
            let json = json_flag || entity_json;
            let user = api_client.get_user().await?;
            match api_client.get_clients(user.default_workspace_id).await {
                Err(error) => println!("{}\n{}", "Couldn't fetch clients from API".red(), error),
                Ok(clients) => {
                    let stdout = io::stdout();
                    let mut handle = BufWriter::new(stdout);
                    let clients = clients
                        .iter()
                        .take(count.unwrap_or(usize::MAX))
                        .collect::<Vec<_>>();
                    if json {
                        let json_string = serde_json::to_string_pretty(&clients)
                            .expect("failed to serialize clients to JSON");
                        writeln!(handle, "{json_string}").expect("failed to print");
                    } else {
                        clients
                            .iter()
                            .for_each(|c| writeln!(handle, "{c}").expect("failed to print"));
                    }
                }
            }
            return Ok(());
        }

        match api_client.get_entities().await {
            Err(error) => println!(
                "{}\n{}",
                "Couldn't fetch time entries the from API".red(),
                error
            ),
            Ok(entities) => {
                // use this to avoid calling println! in a loop:
                // <https://rust-cli.github.io/book/tutorial/output.html#a-note-on-printing-performance>
                let stdout = io::stdout();
                let mut handle = BufWriter::new(stdout);

                // TODO: better error handling for writeln!
                match entity.unwrap_or(Entity::TimeEntry { json: false }) {
                    Entity::TimeEntry { json: entity_json } => {
                        let json = json_flag || entity_json;
                        let entries = entities
                            .time_entries
                            .iter()
                            .take(count.unwrap_or(usize::MAX))
                            .collect::<Vec<_>>();

                        if json {
                            let json_string = serde_json::to_string_pretty(&entries)
                                .expect("failed to serialize time entries to JSON");
                            writeln!(handle, "{json_string}").expect("failed to print");
                        } else {
                            entries.iter().for_each(|time_entry| {
                                writeln!(handle, "{time_entry}").expect("failed to print")
                            });
                        }
                    }

                    Entity::Project { json: entity_json } => {
                        let json = json_flag || entity_json;
                        let projects = entities
                            .projects
                            .values()
                            .take(count.unwrap_or(usize::MAX))
                            .collect::<Vec<_>>();

                        if json {
                            let json_string = serde_json::to_string_pretty(&projects)
                                .expect("failed to serialize projects to JSON");
                            writeln!(handle, "{json_string}").expect("failed to print");
                        } else {
                            projects.iter().for_each(|project| {
                                writeln!(handle, "{project}").expect("failed to print")
                            });
                        }
                    }

                    Entity::Workspace { json: entity_json } => {
                        let json = json_flag || entity_json;
                        let workspaces = entities
                            .workspaces
                            .iter()
                            .take(count.unwrap_or(usize::MAX))
                            .collect::<Vec<_>>();

                        if json {
                            let json_string = serde_json::to_string_pretty(&workspaces)
                                .expect("failed to serialize workspaces to JSON");
                            writeln!(handle, "{json_string}").expect("failed to print");
                        } else {
                            workspaces.iter().for_each(|workspace| {
                                writeln!(handle, "{workspace}").expect("failed to print")
                            });
                        }
                    }

                    Entity::Task { json: entity_json } => {
                        let json = json_flag || entity_json;
                        let tasks = entities
                            .tasks
                            .values()
                            .take(count.unwrap_or(usize::MAX))
                            .collect::<Vec<_>>();

                        if json {
                            let json_string = serde_json::to_string_pretty(&tasks)
                                .expect("failed to serialize tasks to JSON");
                            writeln!(handle, "{json_string}").expect("failed to print");
                        } else {
                            tasks.iter().for_each(|task| {
                                writeln!(handle, "{task}").expect("failed to print")
                            });
                        }
                    }

                    // Already handled above via dedicated API paths
                    Entity::Tag { .. } | Entity::Client { .. } => unreachable!(),
                };
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::client::MockApiClient;
    use crate::models::{Client, Entities, Tag, TimeEntry, User};
    use chrono::Utc;
    use std::collections::HashMap;
    use tokio_test::assert_ok;

    fn mock_user() -> User {
        User {
            api_token: "token".to_string(),
            email: "test@example.com".to_string(),
            fullname: Some("Test".to_string()),
            timezone: "UTC".to_string(),
            default_workspace_id: 1,
            beginning_of_week: None,
            image_url: None,
            created_at: None,
            updated_at: None,
            country_id: None,
            has_password: None,
        }
    }

    fn mock_time_entry() -> TimeEntry {
        TimeEntry {
            id: 42,
            description: "Test entry".to_string(),
            start: Utc::now(),
            stop: None,
            duration: -Utc::now().timestamp(),
            billable: false,
            workspace_id: 1,
            tags: vec!["dev".to_string()],
            project: None,
            task: None,
            created_with: Some("toggl-cli".to_string()),
        }
    }

    fn mock_entities() -> Entities {
        Entities {
            time_entries: vec![mock_time_entry()],
            projects: HashMap::new(),
            tasks: HashMap::new(),
            clients: HashMap::new(),
            workspaces: Vec::new(),
            tags: Vec::new(),
        }
    }

    #[tokio::test]
    async fn list_time_entries_with_date_filter_uses_filtered_endpoint() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_time_entries_filtered()
            .withf(|since, until| {
                since.as_deref() == Some("2026-01-01") && until.as_deref() == Some("2026-01-31")
            })
            .returning(|_, _| Ok(vec![mock_time_entry()]));

        let result = ListCommand::execute(
            api_client,
            Some(1),
            false,
            Some("2026-01-01".to_string()),
            Some("2026-01-31".to_string()),
            None,
        )
        .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn list_tags_uses_workspace_specific_api() {
        let mut api_client = MockApiClient::new();
        let user = mock_user();
        api_client
            .expect_get_user()
            .returning(move || Ok(user.clone()));
        api_client.expect_get_tags().withf(|wid| *wid == 1).returning(|wid| {
            Ok(vec![Tag {
                id: 10,
                name: "backend".to_string(),
                workspace_id: wid,
            }])
        });

        let result = ListCommand::execute(
            api_client,
            None,
            false,
            None,
            None,
            Some(Entity::Tag { json: false }),
        )
        .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn list_clients_uses_workspace_specific_api() {
        let mut api_client = MockApiClient::new();
        let user = mock_user();
        api_client
            .expect_get_user()
            .returning(move || Ok(user.clone()));
        api_client
            .expect_get_clients()
            .withf(|wid| *wid == 1)
            .returning(|wid| {
                Ok(vec![Client {
                    id: 20,
                    name: "Acme".to_string(),
                    workspace_id: wid,
                }])
            });

        let result = ListCommand::execute(
            api_client,
            None,
            false,
            None,
            None,
            Some(Entity::Client { json: true }),
        )
        .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn list_projects_uses_entities_snapshot() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_entities()
            .returning(|| Ok(mock_entities()));

        let result = ListCommand::execute(
            api_client,
            Some(1),
            false,
            None,
            None,
            Some(Entity::Project { json: false }),
        )
        .await;
        assert_ok!(result);
    }
}
