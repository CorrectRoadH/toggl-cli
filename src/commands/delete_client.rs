use crate::api::client::ApiClient;
use crate::models::ResultWithDefaultError;
use colored::Colorize;

pub struct DeleteClientCommand;

impl DeleteClientCommand {
    pub async fn execute(api_client: impl ApiClient, name: String) -> ResultWithDefaultError<()> {
        let workspace_id = api_client.get_user().await?.default_workspace_id;
        let clients = api_client.get_clients(workspace_id).await?;

        let client = clients.into_iter().find(|c| c.name == name);

        match client {
            None => println!("{}", format!("No client found with name '{name}'").yellow()),
            Some(client) => match api_client.delete_client(workspace_id, client.id).await {
                Err(error) => println!("{}\n{}", "Couldn't delete client".red(), error),
                Ok(()) => println!("{}", "Client deleted successfully".green()),
            },
        }

        Ok(())
    }
}
