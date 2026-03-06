use crate::api::client::ApiClient;
use crate::error::ArgumentError;
use crate::models::ResultWithDefaultError;

pub struct DeleteProjectCommand;

impl DeleteProjectCommand {
    pub async fn execute(api_client: impl ApiClient, name: String) -> ResultWithDefaultError<()> {
        let project = api_client
            .get_projects_list()
            .await?
            .into_iter()
            .find(|p| p.name == name)
            .ok_or_else(|| {
                Box::new(ArgumentError::ResourceNotFound(format!(
                    "No project found with name '{name}'"
                ))) as Box<dyn std::error::Error + Send>
            })?;

        api_client
            .delete_project(project.workspace_id, project.id)
            .await?;
        println!("Project deleted successfully\n{}", project);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::client::MockApiClient;
    use crate::error::ApiError;
    use crate::models::Project;
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
    async fn delete_project_returns_ok_on_success() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_projects_list()
            .returning(|| Ok(mock_projects()));
        api_client
            .expect_delete_project()
            .withf(|wid, pid| *wid == 1 && *pid == 10)
            .returning(|_, _| Ok(()));

        let result = DeleteProjectCommand::execute(api_client, "Platform".to_string()).await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn delete_project_handles_not_found() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_projects_list()
            .returning(|| Ok(mock_projects()));

        let result = DeleteProjectCommand::execute(api_client, "Missing".to_string()).await;
        assert_err!(result);
    }

    #[tokio::test]
    async fn delete_project_handles_api_failure() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_projects_list()
            .returning(|| Ok(mock_projects()));
        api_client
            .expect_delete_project()
            .returning(|_, _| Err(Box::new(ApiError::Network)));

        let result = DeleteProjectCommand::execute(api_client, "Platform".to_string()).await;
        assert_err!(result);
    }
}
