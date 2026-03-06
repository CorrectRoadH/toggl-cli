use crate::api::client::ApiClient;
use crate::models::ResultWithDefaultError;
pub struct CreateTagCommand;

impl CreateTagCommand {
    pub async fn execute(api_client: impl ApiClient, name: String) -> ResultWithDefaultError<()> {
        let workspace_id = api_client.get_user().await?.default_workspace_id;
        let tag = api_client.create_tag(workspace_id, name).await?;
        println!("Tag created successfully\n{}", tag);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::client::MockApiClient;
    use crate::error::ApiError;
    use crate::models::{Tag, User};
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
    async fn create_tag_returns_ok_on_success() {
        let mut api_client = MockApiClient::new();
        let user = mock_user();
        api_client
            .expect_get_user()
            .returning(move || Ok(user.clone()));
        api_client
            .expect_create_tag()
            .withf(|wid, name| *wid == 1 && name == "backend")
            .returning(|wid, name| {
                Ok(Tag {
                    id: 100,
                    name,
                    workspace_id: wid,
                })
            });

        let result = CreateTagCommand::execute(api_client, "backend".to_string()).await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn create_tag_returns_error_on_api_failure() {
        let mut api_client = MockApiClient::new();
        let user = mock_user();
        api_client
            .expect_get_user()
            .returning(move || Ok(user.clone()));
        api_client
            .expect_create_tag()
            .returning(|_, _| Err(Box::new(ApiError::Network)));

        let result = CreateTagCommand::execute(api_client, "backend".to_string()).await;
        assert_err!(result);
    }

    #[tokio::test]
    async fn create_tag_returns_error_when_user_fetch_fails() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_user()
            .returning(|| Err(Box::new(ApiError::Network)));

        let result = CreateTagCommand::execute(api_client, "backend".to_string()).await;
        assert_err!(result);
    }
}
