use crate::api::client::ApiClient;
use crate::error::ArgumentError;
use crate::models::ResultWithDefaultError;

pub struct DeleteTagCommand;

impl DeleteTagCommand {
    pub async fn execute(api_client: impl ApiClient, name: String) -> ResultWithDefaultError<()> {
        let workspace_id = api_client.get_user().await?.default_workspace_id;
        let tags = api_client.get_tags(workspace_id).await?;

        let tag = tags.into_iter().find(|t| t.name == name).ok_or_else(|| {
            Box::new(ArgumentError::ResourceNotFound(format!(
                "No tag found with name '{name}'"
            ))) as Box<dyn std::error::Error + Send>
        })?;

        api_client.delete_tag(workspace_id, tag.id).await?;
        println!("Tag deleted successfully");

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

    fn mock_tags() -> Vec<Tag> {
        vec![
            Tag {
                id: 10,
                name: "backend".to_string(),
                workspace_id: 1,
            },
            Tag {
                id: 20,
                name: "frontend".to_string(),
                workspace_id: 1,
            },
        ]
    }

    #[tokio::test]
    async fn delete_tag_returns_ok_on_success() {
        let mut api_client = MockApiClient::new();
        let user = mock_user();
        api_client
            .expect_get_user()
            .returning(move || Ok(user.clone()));
        api_client.expect_get_tags().returning(|_| Ok(mock_tags()));
        api_client
            .expect_delete_tag()
            .withf(|wid, tag_id| *wid == 1 && *tag_id == 10)
            .returning(|_, _| Ok(()));

        let result = DeleteTagCommand::execute(api_client, "backend".to_string()).await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn delete_tag_handles_not_found() {
        let mut api_client = MockApiClient::new();
        let user = mock_user();
        api_client
            .expect_get_user()
            .returning(move || Ok(user.clone()));
        api_client.expect_get_tags().returning(|_| Ok(mock_tags()));

        let result = DeleteTagCommand::execute(api_client, "missing".to_string()).await;
        assert_err!(result);
    }

    #[tokio::test]
    async fn delete_tag_handles_api_failure() {
        let mut api_client = MockApiClient::new();
        let user = mock_user();
        api_client
            .expect_get_user()
            .returning(move || Ok(user.clone()));
        api_client.expect_get_tags().returning(|_| Ok(mock_tags()));
        api_client
            .expect_delete_tag()
            .returning(|_, _| Err(Box::new(ApiError::Network)));

        let result = DeleteTagCommand::execute(api_client, "backend".to_string()).await;
        assert_err!(result);
    }
}
