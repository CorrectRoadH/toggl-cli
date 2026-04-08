use crate::api::client::ApiClient;
use crate::error::ArgumentError;
use crate::models::ResultWithDefaultError;

pub struct UpdateTaskCommand;

impl UpdateTaskCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        project_name: String,
        task_name: String,
        new_name: Option<String>,
        active: Option<bool>,
        estimated_seconds: Option<i64>,
        user_id: Option<i64>,
    ) -> ResultWithDefaultError<()> {
        if new_name.is_none()
            && active.is_none()
            && estimated_seconds.is_none()
            && user_id.is_none()
        {
            return Err(Box::new(ArgumentError::MissingUpdateFields(
                "Provide at least one of --new-name, --active, --estimated-seconds, or --user-id"
                    .to_string(),
            )));
        }

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

        let task = api_client
            .update_task(
                task.workspace_id,
                task.project.id,
                task.id,
                new_name,
                active,
                estimated_seconds,
                user_id,
            )
            .await?;
        println!("Task updated successfully\n{}", task);

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
            active: true,
        }
    }

    fn mock_tasks() -> Vec<Task> {
        vec![mock_task()]
    }

    #[tokio::test]
    async fn update_task_returns_ok_on_success() {
        let mut api_client = MockApiClient::new();
        let mut updated_task = mock_task();
        updated_task.name = "Review v2".to_string();
        api_client
            .expect_get_tasks_list()
            .returning(|| Ok(mock_tasks()));
        api_client
            .expect_update_task()
            .withf(|wid, pid, tid, name, active, estimate, user_id| {
                *wid == 1
                    && *pid == 10
                    && *tid == 77
                    && name.as_deref() == Some("Review v2")
                    && *active == Some(false)
                    && *estimate == Some(1800)
                    && *user_id == Some(8)
            })
            .returning(move |_, _, _, _, _, _, _| Ok(updated_task.clone()));

        let result = UpdateTaskCommand::execute(
            api_client,
            "Platform".to_string(),
            "Review".to_string(),
            Some("Review v2".to_string()),
            Some(false),
            Some(1800),
            Some(8),
        )
        .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn update_task_handles_missing_project() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_tasks_list()
            .returning(|| Ok(mock_tasks()));

        let result = UpdateTaskCommand::execute(
            api_client,
            "Missing".to_string(),
            "Review".to_string(),
            Some("Review v2".to_string()),
            None,
            None,
            None,
        )
        .await;
        assert_err!(result);
    }

    #[tokio::test]
    async fn update_task_handles_missing_task() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_tasks_list()
            .returning(|| Ok(mock_tasks()));

        let result = UpdateTaskCommand::execute(
            api_client,
            "Platform".to_string(),
            "Missing".to_string(),
            Some("Review v2".to_string()),
            None,
            None,
            None,
        )
        .await;
        assert_err!(result);
    }

    #[tokio::test]
    async fn update_task_handles_api_failure() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_tasks_list()
            .returning(|| Ok(mock_tasks()));
        api_client
            .expect_update_task()
            .returning(|_, _, _, _, _, _, _| Err(Box::new(ApiError::Network)));

        let result = UpdateTaskCommand::execute(
            api_client,
            "Platform".to_string(),
            "Review".to_string(),
            Some("Review v2".to_string()),
            None,
            None,
            None,
        )
        .await;
        assert_err!(result);
    }

    #[tokio::test]
    async fn update_task_returns_error_without_update_fields() {
        let api_client = MockApiClient::new();

        let result = UpdateTaskCommand::execute(
            api_client,
            "Platform".to_string(),
            "Review".to_string(),
            None,
            None,
            None,
            None,
        )
        .await;
        assert_err!(result);
    }
}
