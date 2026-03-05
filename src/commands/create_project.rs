use crate::api;
use crate::models;
use api::client::ApiClient;
use colored::Colorize;
use models::ResultWithDefaultError;

pub struct CreateProjectCommand;

impl CreateProjectCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        name: String,
        color: String,
    ) -> ResultWithDefaultError<()> {
        let user = api_client.get_user().await?;
        let workspace_id = user.default_workspace_id;

        match api_client.create_project(workspace_id, name, color).await {
            Err(error) => println!("{}\n{}", "Couldn't create project".red(), error),
            Ok(project) => println!("{} {}", "Created project:".green(), project),
        }

        Ok(())
    }
}
