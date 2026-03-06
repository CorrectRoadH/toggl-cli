use crate::api::client::ApiClient;
use crate::models::ResultWithDefaultError;
use colored::Colorize;

pub struct CreateClientCommand;

impl CreateClientCommand {
    pub async fn execute(api_client: impl ApiClient, name: String) -> ResultWithDefaultError<()> {
        let workspace_id = api_client.get_user().await?.default_workspace_id;
        match api_client.create_client(workspace_id, name).await {
            Err(error) => println!("{}\n{}", "Couldn't create client".red(), error),
            Ok(client) => println!("{}\n{}", "Client created successfully".green(), client),
        }
        Ok(())
    }
}
