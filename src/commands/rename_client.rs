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
