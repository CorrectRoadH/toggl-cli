use crate::api::client::ApiClient;
use crate::error::ArgumentError;
use crate::models::ResultWithDefaultError;

pub struct RenameTagCommand;

impl RenameTagCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        old_name: String,
        new_name: String,
    ) -> ResultWithDefaultError<()> {
        let workspace_id = api_client.get_user().await?.default_workspace_id;
        let tags = api_client.get_tags(workspace_id).await?;

        let tag = tags
            .into_iter()
            .find(|t| t.name == old_name)
            .ok_or_else(|| {
                Box::new(ArgumentError::ResourceNotFound(format!(
                    "No tag found with name '{old_name}'"
                ))) as Box<dyn std::error::Error + Send>
            })?;

        let tag = api_client
            .rename_tag(workspace_id, tag.id, new_name)
            .await?;
        println!("Tag renamed successfully\n{}", tag);

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
        vec![Tag {
            id: 10,
            name: "OldName".to_string(),
            workspace_id: 1,
        }]
    }

    #[tokio::test]
    async fn rename_tag_returns_ok_on_success() {
        let mut api_client = MockApiClient::new();
        let user = mock_user();
        api_client
            .expect_get_user()
            .returning(move || Ok(user.clone()));
        api_client.expect_get_tags().returning(|_| Ok(mock_tags()));
        api_client
            .expect_rename_tag()
            .withf(|wid, tag_id, new_name| *wid == 1 && *tag_id == 10 && new_name == "NewName")
            .returning(|wid, _, name| {
                Ok(Tag {
                    id: 10,
                    name,
                    workspace_id: wid,
                })
            });

        let result =
            RenameTagCommand::execute(api_client, "OldName".to_string(), "NewName".to_string())
                .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn rename_tag_handles_not_found() {
        let mut api_client = MockApiClient::new();
        let user = mock_user();
        api_client
            .expect_get_user()
            .returning(move || Ok(user.clone()));
        api_client.expect_get_tags().returning(|_| Ok(mock_tags()));

        let result =
            RenameTagCommand::execute(api_client, "NonExistent".to_string(), "NewName".to_string())
                .await;
        assert_err!(result);
    }

    #[tokio::test]
    async fn rename_tag_handles_api_failure() {
        let mut api_client = MockApiClient::new();
        let user = mock_user();
        api_client
            .expect_get_user()
            .returning(move || Ok(user.clone()));
        api_client.expect_get_tags().returning(|_| Ok(mock_tags()));
        api_client
            .expect_rename_tag()
            .returning(|_, _, _| Err(Box::new(ApiError::Network)));

        let result =
            RenameTagCommand::execute(api_client, "OldName".to_string(), "NewName".to_string())
                .await;
        assert_err!(result);
    }
}
