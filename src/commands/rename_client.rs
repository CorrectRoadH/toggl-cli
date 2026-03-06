use crate::api::client::ApiClient;
use crate::models::ResultWithDefaultError;
use colored::Colorize;

pub struct RenameClientCommand;

impl RenameClientCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        old_name: String,
        new_name: String,
    ) -> ResultWithDefaultError<()> {
        let workspace_id = api_client.get_user().await?.default_workspace_id;
        let clients = api_client.get_clients(workspace_id).await?;

        let client = clients.into_iter().find(|c| c.name == old_name);

        match client {
            None => println!(
                "{}",
                format!("No client found with name '{old_name}'").yellow()
            ),
            Some(client) => {
                match api_client
                    .rename_client(workspace_id, client.id, new_name)
                    .await
                {
                    Err(error) => println!("{}\n{}", "Couldn't rename client".red(), error),
                    Ok(client) => {
                        println!("{}\n{}", "Client renamed successfully".green(), client)
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::client::MockApiClient;
    use crate::error::ApiError;
    use crate::models::{Client, User};
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

    fn mock_clients() -> Vec<Client> {
        vec![Client {
            id: 10,
            name: "OldName".to_string(),
            workspace_id: 1,
        }]
    }

    #[tokio::test]
    async fn rename_client_returns_ok_on_success() {
        let mut api_client = MockApiClient::new();
        let user = mock_user();
        api_client
            .expect_get_user()
            .returning(move || Ok(user.clone()));
        api_client
            .expect_get_clients()
            .returning(|_| Ok(mock_clients()));
        api_client
            .expect_rename_client()
            .withf(|wid, cid, name| *wid == 1 && *cid == 10 && name == "NewName")
            .returning(|wid, _, name| {
                Ok(Client {
                    id: 10,
                    name,
                    workspace_id: wid,
                })
            });

        let result =
            RenameClientCommand::execute(api_client, "OldName".to_string(), "NewName".to_string())
                .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn rename_client_handles_not_found() {
        let mut api_client = MockApiClient::new();
        let user = mock_user();
        api_client
            .expect_get_user()
            .returning(move || Ok(user.clone()));
        api_client
            .expect_get_clients()
            .returning(|_| Ok(mock_clients()));

        let result = RenameClientCommand::execute(
            api_client,
            "NonExistent".to_string(),
            "NewName".to_string(),
        )
        .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn rename_client_handles_api_failure() {
        let mut api_client = MockApiClient::new();
        let user = mock_user();
        api_client
            .expect_get_user()
            .returning(move || Ok(user.clone()));
        api_client
            .expect_get_clients()
            .returning(|_| Ok(mock_clients()));
        api_client
            .expect_rename_client()
            .returning(|_, _, _| Err(Box::new(ApiError::Network)));

        let result =
            RenameClientCommand::execute(api_client, "OldName".to_string(), "NewName".to_string())
                .await;
        assert_ok!(result);
    }
}
