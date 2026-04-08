use crate::api::client::ApiClient;
use crate::error::ArgumentError;
use crate::models::ResultWithDefaultError;

pub struct CreateTaskCommand;

impl CreateTaskCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        project_name: String,
        name: String,
        active: Option<bool>,
        estimated_seconds: Option<i64>,
        user_id: Option<i64>,
    ) -> ResultWithDefaultError<()> {
        let project = api_client
            .get_projects_list()
            .await?
            .into_iter()
            .find(|project| project.name == project_name)
            .ok_or_else(|| {
                Box::new(ArgumentError::ResourceNotFound(format!(
                    "No project found with name '{project_name}'"
                ))) as Box<dyn std::error::Error + Send>
            })?;

        let task = api_client
            .create_task(
                project.workspace_id,
                project.id,
                name,
                active,
                estimated_seconds,
                user_id,
            )
            .await?;
        println!("Task created successfully\n{}", task);

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

    fn mock_projects() -> Vec<Project> {
        vec![mock_project()]
    }

    #[tokio::test]
    async fn create_task_returns_ok_on_success() {
        let mut api_client = MockApiClient::new();
        let project = mock_project();
        api_client
            .expect_get_projects_list()
            .returning(|| Ok(mock_projects()));
        api_client
            .expect_create_task()
            .withf(|wid, pid, name, active, estimate, user_id| {
                *wid == 1
                    && *pid == 10
                    && name == "Review"
                    && *active == Some(true)
                    && *estimate == Some(3600)
                    && *user_id == Some(5)
            })
            .returning(move |_, _, name, _, _, _| {
                Ok(Task {
                    id: 99,
                    name,
                    project: project.clone(),
                    workspace_id: 1,
                    active: true,
                })
            });

        let result = CreateTaskCommand::execute(
            api_client,
            "Platform".to_string(),
            "Review".to_string(),
            Some(true),
            Some(3600),
            Some(5),
        )
        .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn create_task_handles_missing_project() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_projects_list()
            .returning(|| Ok(mock_projects()));

        let result = CreateTaskCommand::execute(
            api_client,
            "Missing".to_string(),
            "Review".to_string(),
            None,
            None,
            None,
        )
        .await;
        assert_err!(result);
    }

    #[tokio::test]
    async fn create_task_handles_api_failure() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_projects_list()
            .returning(|| Ok(mock_projects()));
        api_client
            .expect_create_task()
            .returning(|_, _, _, _, _, _| Err(Box::new(ApiError::Network)));

        let result = CreateTaskCommand::execute(
            api_client,
            "Platform".to_string(),
            "Review".to_string(),
            None,
            None,
            None,
        )
        .await;
        assert_err!(result);
    }

    #[tokio::test]
    async fn create_task_returns_error_when_projects_fetch_fails() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_projects_list()
            .returning(|| Err(Box::new(ApiError::Network)));

        let result = CreateTaskCommand::execute(
            api_client,
            "Platform".to_string(),
            "Review".to_string(),
            None,
            None,
            None,
        )
        .await;
        assert_err!(result);
    }
}
