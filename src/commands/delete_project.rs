use crate::api::client::ApiClient;
use crate::models::ResultWithDefaultError;
use colored::Colorize;

pub struct DeleteProjectCommand;

impl DeleteProjectCommand {
    pub async fn execute(api_client: impl ApiClient, name: String) -> ResultWithDefaultError<()> {
        let workspace_id = api_client.get_user().await?.default_workspace_id;
        let entities = api_client.get_entities().await?;

        let project = entities.projects.values().find(|p| p.name == name).cloned();

        match project {
            None => println!(
                "{}",
                format!("No project found with name '{name}'").yellow()
            ),
            Some(project) => match api_client.delete_project(workspace_id, project.id).await {
                Err(error) => println!("{}\n{}", "Couldn't delete project".red(), error),
                Ok(()) => println!("{}\n{}", "Project deleted successfully".green(), project),
            },
        }

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
    async fn delete_project_returns_ok_on_success() {
        let mut api_client = MockApiClient::new();
        let user = mock_user();
        api_client
            .expect_get_user()
            .returning(move || Ok(user.clone()));
        api_client
            .expect_get_entities()
            .returning(|| Ok(mock_entities()));
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
        let user = mock_user();
        api_client
            .expect_get_user()
            .returning(move || Ok(user.clone()));
        api_client
            .expect_get_entities()
            .returning(|| Ok(mock_entities()));

        let result = DeleteProjectCommand::execute(api_client, "Missing".to_string()).await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn delete_project_handles_api_failure() {
        let mut api_client = MockApiClient::new();
        let user = mock_user();
        api_client
            .expect_get_user()
            .returning(move || Ok(user.clone()));
        api_client
            .expect_get_entities()
            .returning(|| Ok(mock_entities()));
        api_client
            .expect_delete_project()
            .returning(|_, _| Err(Box::new(ApiError::Network)));

        let result = DeleteProjectCommand::execute(api_client, "Platform".to_string()).await;
        assert_ok!(result);
    }
}
