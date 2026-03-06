use crate::api::client::ApiClient;
use crate::models::ResultWithDefaultError;
use colored::Colorize;

pub struct CreateWorkspaceCommand;

impl CreateWorkspaceCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        organization_id: i64,
        name: String,
    ) -> ResultWithDefaultError<()> {
        match api_client.create_workspace(organization_id, name).await {
            Err(error) => println!("{}\n{}", "Couldn't create workspace".red(), error),
            Ok(workspace) => println!(
                "{}\n{}",
                "Workspace created successfully".green(),
                workspace
            ),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::client::MockApiClient;
    use crate::error::ApiError;
    use crate::models::Workspace;
    use tokio_test::assert_ok;

    #[tokio::test]
    async fn create_workspace_returns_ok_on_success() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_create_workspace()
            .withf(|organization_id, name| *organization_id == 42 && name == "Platform")
            .returning(|_, name| {
                Ok(Workspace {
                    id: 100,
                    name,
                    admin: true,
                })
            });

        let result = CreateWorkspaceCommand::execute(api_client, 42, "Platform".to_string()).await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn create_workspace_returns_ok_on_api_failure() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_create_workspace()
            .returning(|_, _| Err(Box::new(ApiError::Network)));

        let result = CreateWorkspaceCommand::execute(api_client, 42, "Platform".to_string()).await;
        assert_ok!(result);
    }
}
