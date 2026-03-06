use crate::api::client::ApiClient;
use crate::models::ResultWithDefaultError;
use colored::Colorize;

pub struct CreateProjectCommand;

impl CreateProjectCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        name: String,
        color: String,
    ) -> ResultWithDefaultError<()> {
        let workspace_id = api_client.get_user().await?.default_workspace_id;
        match api_client.create_project(workspace_id, name, color).await {
            Err(error) => println!("{}\n{}", "Couldn't create project".red(), error),
            Ok(project) => println!("{}\n{}", "Project created successfully".green(), project),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::client::MockApiClient;
    use crate::error::ApiError;
    use crate::models::{Project, User};
    use chrono::Utc;
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

    #[tokio::test]
    async fn create_project_returns_ok_on_success() {
        let mut api_client = MockApiClient::new();
        let user = mock_user();
        api_client
            .expect_get_user()
            .returning(move || Ok(user.clone()));
        api_client
            .expect_create_project()
            .withf(|wid, name, color| *wid == 1 && name == "Platform" && color == "#06aaf5")
            .returning(|wid, name, color| {
                Ok(Project {
                    id: 100,
                    name,
                    workspace_id: wid,
                    client: None,
                    is_private: false,
                    active: true,
                    at: Utc::now(),
                    created_at: Utc::now(),
                    color,
                    billable: None,
                })
            });

        let result = CreateProjectCommand::execute(
            api_client,
            "Platform".to_string(),
            "#06aaf5".to_string(),
        )
        .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn create_project_returns_ok_on_api_failure() {
        let mut api_client = MockApiClient::new();
        let user = mock_user();
        api_client
            .expect_get_user()
            .returning(move || Ok(user.clone()));
        api_client
            .expect_create_project()
            .returning(|_, _, _| Err(Box::new(ApiError::Network)));

        let result =
            CreateProjectCommand::execute(api_client, "Fail".to_string(), "#000000".to_string())
                .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn create_project_returns_error_when_user_fetch_fails() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_user()
            .returning(|| Err(Box::new(ApiError::Network)));

        let result = CreateProjectCommand::execute(
            api_client,
            "Platform".to_string(),
            "#06aaf5".to_string(),
        )
        .await;
        assert_err!(result);
    }
}
