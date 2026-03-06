use crate::api::client::ApiClient;
use crate::error::ArgumentError;
use crate::models::ResultWithDefaultError;

pub struct RenameProjectCommand;

impl RenameProjectCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        old_name: String,
        new_name: String,
    ) -> ResultWithDefaultError<()> {
        let project = api_client
            .get_projects_list()
            .await?
            .into_iter()
            .find(|p| p.name == old_name)
            .ok_or_else(|| {
                Box::new(ArgumentError::ResourceNotFound(format!(
                    "No project found with name '{old_name}'"
                ))) as Box<dyn std::error::Error + Send>
            })?;

        let project = api_client
            .rename_project(project.workspace_id, project.id, new_name)
            .await?;
        println!("Project renamed successfully\n{}", project);

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
            name: "OldName".to_string(),
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
    async fn rename_project_returns_ok_on_success() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_projects_list()
            .returning(|| Ok(mock_projects()));
        api_client
            .expect_rename_project()
            .withf(|wid, pid, name| *wid == 1 && *pid == 10 && name == "NewName")
            .returning(|wid, _, name| {
                Ok(Project {
                    id: 10,
                    name,
                    workspace_id: wid,
                    client: None,
                    is_private: false,
                    active: true,
                    at: Utc::now(),
                    created_at: Utc::now(),
                    color: "#06aaf5".to_string(),
                    billable: None,
                })
            });

        let result =
            RenameProjectCommand::execute(api_client, "OldName".to_string(), "NewName".to_string())
                .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn rename_project_handles_not_found() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_projects_list()
            .returning(|| Ok(mock_projects()));

        let result =
            RenameProjectCommand::execute(api_client, "Missing".to_string(), "NewName".to_string())
                .await;
        assert_err!(result);
    }

    #[tokio::test]
    async fn rename_project_handles_api_failure() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_projects_list()
            .returning(|| Ok(mock_projects()));
        api_client
            .expect_rename_project()
            .returning(|_, _, _| Err(Box::new(ApiError::Network)));

        let result =
            RenameProjectCommand::execute(api_client, "OldName".to_string(), "NewName".to_string())
                .await;
        assert_err!(result);
    }
}
