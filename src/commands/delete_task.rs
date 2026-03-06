use crate::api::client::ApiClient;
use crate::error::ArgumentError;
use crate::models::ResultWithDefaultError;

pub struct DeleteTaskCommand;

impl DeleteTaskCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        project_name: String,
        task_name: String,
    ) -> ResultWithDefaultError<()> {
        let task = api_client
            .get_tasks_list()
            .await?
            .into_iter()
            .find(|task| task.name == task_name && task.project.name == project_name)
            .ok_or_else(|| {
                Box::new(ArgumentError::ResourceNotFound(format!(
                    "No task found with name '{task_name}' in project '{}'",
                    project_name
                ))) as Box<dyn std::error::Error + Send>
            })?;

        api_client
            .delete_task(task.workspace_id, task.project.id, task.id)
            .await?;
        println!("Task deleted successfully\n{}", task);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::client::MockApiClient;
    use crate::error::ApiError;
    use crate::models::{Project, Task};
    use chrono::Utc;
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

    fn mock_tasks() -> Vec<Task> {
        vec![mock_task()]
    }

    #[tokio::test]
    async fn delete_task_returns_ok_on_success() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_tasks_list()
            .returning(|| Ok(mock_tasks()));
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
            .expect_get_tasks_list()
            .returning(|| Ok(mock_tasks()));

        let result =
            DeleteTaskCommand::execute(api_client, "Missing".to_string(), "Review".to_string())
                .await;
        assert_err!(result);
    }

    #[tokio::test]
    async fn delete_task_handles_missing_task() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_tasks_list()
            .returning(|| Ok(mock_tasks()));

        let result =
            DeleteTaskCommand::execute(api_client, "Platform".to_string(), "Missing".to_string())
                .await;
        assert_err!(result);
    }

    #[tokio::test]
    async fn delete_task_handles_api_failure() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_tasks_list()
            .returning(|| Ok(mock_tasks()));
        api_client
            .expect_delete_task()
            .returning(|_, _, _| Err(Box::new(ApiError::Network)));

        let result =
            DeleteTaskCommand::execute(api_client, "Platform".to_string(), "Review".to_string())
                .await;
        assert_err!(result);
    }

    #[tokio::test]
    async fn delete_task_returns_error_when_tasks_fetch_fails() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_tasks_list()
            .returning(|| Err(Box::new(ApiError::Network)));

        let result =
            DeleteTaskCommand::execute(api_client, "Platform".to_string(), "Review".to_string())
                .await;
        assert_err!(result);
    }
}
