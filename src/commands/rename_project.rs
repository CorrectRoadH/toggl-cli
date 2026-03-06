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
        let workspace_id = api_client.get_user().await?.default_workspace_id;
        let entities = api_client.get_entities().await?;

        let project = entities
            .projects
            .values()
            .find(|p| p.name == old_name)
            .cloned()
            .ok_or_else(|| {
                Box::new(ArgumentError::ResourceNotFound(format!(
                    "No project found with name '{old_name}'"
                ))) as Box<dyn std::error::Error + Send>
            })?;

        let project = api_client
            .rename_project(workspace_id, project.id, new_name)
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
    use crate::models::{Entities, Project, User};
    use chrono::Utc;
    use std::collections::HashMap;
    use tokio_test::{assert_err, assert_ok};

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

    fn mock_entities() -> Entities {
        let project = mock_project();
        let mut projects = HashMap::new();
        projects.insert(project.id, project);
        Entities {
            time_entries: Vec::new(),
            projects,
            tasks: HashMap::new(),
            clients: HashMap::new(),
            workspaces: Vec::new(),
            tags: Vec::new(),
        }
    }

    #[tokio::test]
    async fn rename_project_returns_ok_on_success() {
        let mut api_client = MockApiClient::new();
        let user = mock_user();
        api_client
            .expect_get_user()
            .returning(move || Ok(user.clone()));
        api_client
            .expect_get_entities()
            .returning(|| Ok(mock_entities()));
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
        let user = mock_user();
        api_client
            .expect_get_user()
            .returning(move || Ok(user.clone()));
        api_client
            .expect_get_entities()
            .returning(|| Ok(mock_entities()));

        let result =
            RenameProjectCommand::execute(api_client, "Missing".to_string(), "NewName".to_string())
                .await;
        assert_err!(result);
    }

    #[tokio::test]
    async fn rename_project_handles_api_failure() {
        let mut api_client = MockApiClient::new();
        let user = mock_user();
        api_client
            .expect_get_user()
            .returning(move || Ok(user.clone()));
        api_client
            .expect_get_entities()
            .returning(|| Ok(mock_entities()));
        api_client
            .expect_rename_project()
            .returning(|_, _, _| Err(Box::new(ApiError::Network)));

        let result =
            RenameProjectCommand::execute(api_client, "OldName".to_string(), "NewName".to_string())
                .await;
        assert_err!(result);
    }
}
