use crate::api::client::ApiClient;
use crate::models::ResultWithDefaultError;
use colored::Colorize;

pub struct RenameWorkspaceCommand;

impl RenameWorkspaceCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        old_name: String,
        new_name: String,
    ) -> ResultWithDefaultError<()> {
        let entities = api_client.get_entities().await?;
        let workspace = entities.workspaces.into_iter().find(|w| w.name == old_name);

        match workspace {
            None => println!(
                "{}",
                format!("No workspace found with name '{old_name}'").yellow()
            ),
            Some(workspace) => match api_client.rename_workspace(workspace.id, new_name).await {
                Err(error) => println!("{}\n{}", "Couldn't rename workspace".red(), error),
                Ok(workspace) => println!(
                    "{}\n{}",
                    "Workspace renamed successfully".green(),
                    workspace
                ),
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
    use crate::models::{Entities, Workspace};
    use std::collections::HashMap;
    use tokio_test::{assert_err, assert_ok};

    fn mock_entities() -> Entities {
        Entities {
            time_entries: Vec::new(),
            projects: HashMap::new(),
            tasks: HashMap::new(),
            clients: HashMap::new(),
            workspaces: vec![Workspace {
                id: 10,
                name: "OldName".to_string(),
                admin: true,
            }],
            tags: Vec::new(),
        }
    }

    #[tokio::test]
    async fn rename_workspace_returns_ok_on_success() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_entities()
            .returning(|| Ok(mock_entities()));
        api_client
            .expect_rename_workspace()
            .withf(|workspace_id, new_name| *workspace_id == 10 && new_name == "NewName")
            .returning(|_, new_name| {
                Ok(Workspace {
                    id: 10,
                    name: new_name,
                    admin: true,
                })
            });

        let result = RenameWorkspaceCommand::execute(
            api_client,
            "OldName".to_string(),
            "NewName".to_string(),
        )
        .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn rename_workspace_handles_not_found() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_entities()
            .returning(|| Ok(mock_entities()));

        let result = RenameWorkspaceCommand::execute(
            api_client,
            "Missing".to_string(),
            "NewName".to_string(),
        )
        .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn rename_workspace_handles_api_failure() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_entities()
            .returning(|| Ok(mock_entities()));
        api_client
            .expect_rename_workspace()
            .returning(|_, _| Err(Box::new(ApiError::Network)));

        let result = RenameWorkspaceCommand::execute(
            api_client,
            "OldName".to_string(),
            "NewName".to_string(),
        )
        .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn rename_workspace_returns_error_when_entities_fetch_fails() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_entities()
            .returning(|| Err(Box::new(ApiError::Network)));

        let result = RenameWorkspaceCommand::execute(
            api_client,
            "OldName".to_string(),
            "NewName".to_string(),
        )
        .await;
        assert_err!(result);
    }
}
