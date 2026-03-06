use crate::api::client::ApiClient;
use crate::models::ResultWithDefaultError;
use colored::Colorize;

pub struct DeleteTaskCommand;

impl DeleteTaskCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        project_name: String,
        task_name: String,
    ) -> ResultWithDefaultError<()> {
        let entities = api_client.get_entities().await?;
        let project = entities
            .projects
            .values()
            .find(|project| project.name == project_name)
            .cloned();

        match project {
            None => println!(
                "{}",
                format!("No project found with name '{project_name}'").yellow()
            ),
            Some(project) => {
                let task = entities
                    .tasks
                    .values()
                    .find(|task| task.name == task_name && task.project.id == project.id)
                    .cloned();

                match task {
                    None => println!(
                        "{}",
                        format!(
                            "No task found with name '{task_name}' in project '{}'",
                            project.name
                        )
                        .yellow()
                    ),
                    Some(task) => {
                        match api_client
                            .delete_task(task.workspace_id, task.project.id, task.id)
                            .await
                        {
                            Err(error) => println!("{}\n{}", "Couldn't delete task".red(), error),
                            Ok(()) => println!("{}\n{}", "Task deleted successfully".green(), task),
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::client::MockApiClient;
    use crate::error::ApiError;
    use crate::models::{Entities, Project, Task};
    use chrono::Utc;
    use std::collections::HashMap;
    use tokio_test::{assert_err, assert_ok};

    fn mock_project() -> Project {
        Project {
            id: 10,
            name: "Platform".to_string(),
            workspace_id: 1,
            client: None,
            is_private: false,
            active: true,
            at: Utc::now(),
            created_at: Utc::now(),
            color: "#06aaf5".to_string(),
            billable: None,
        }
    }

    fn mock_task() -> Task {
        Task {
            id: 77,
            name: "Review".to_string(),
            project: mock_project(),
            workspace_id: 1,
        }
    }

    fn mock_entities() -> Entities {
        let project = mock_project();
        let task = mock_task();
        let mut projects = HashMap::new();
        let mut tasks = HashMap::new();
        projects.insert(project.id, project);
        tasks.insert(task.id, task);
        Entities {
            time_entries: Vec::new(),
            projects,
            tasks,
            clients: HashMap::new(),
            workspaces: Vec::new(),
            tags: Vec::new(),
        }
    }

    #[tokio::test]
    async fn delete_task_returns_ok_on_success() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_entities()
            .returning(|| Ok(mock_entities()));
        api_client
            .expect_delete_task()
            .withf(|wid, pid, tid| *wid == 1 && *pid == 10 && *tid == 77)
            .returning(|_, _, _| Ok(()));

        let result =
            DeleteTaskCommand::execute(api_client, "Platform".to_string(), "Review".to_string())
                .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn delete_task_handles_missing_project() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_entities()
            .returning(|| Ok(mock_entities()));

        let result =
            DeleteTaskCommand::execute(api_client, "Missing".to_string(), "Review".to_string())
                .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn delete_task_handles_missing_task() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_entities()
            .returning(|| Ok(mock_entities()));

        let result =
            DeleteTaskCommand::execute(api_client, "Platform".to_string(), "Missing".to_string())
                .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn delete_task_handles_api_failure() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_entities()
            .returning(|| Ok(mock_entities()));
        api_client
            .expect_delete_task()
            .returning(|_, _, _| Err(Box::new(ApiError::Network)));

        let result =
            DeleteTaskCommand::execute(api_client, "Platform".to_string(), "Review".to_string())
                .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn delete_task_returns_error_when_entities_fetch_fails() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_entities()
            .returning(|| Err(Box::new(ApiError::Network)));

        let result =
            DeleteTaskCommand::execute(api_client, "Platform".to_string(), "Review".to_string())
                .await;
        assert_err!(result);
    }
}
