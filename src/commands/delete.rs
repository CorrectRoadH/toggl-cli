use crate::api::client::ApiClient;
use crate::models::ResultWithDefaultError;
use colored::Colorize;

pub struct DeleteCommand;

impl DeleteCommand {
    pub async fn execute(api_client: impl ApiClient, id: i64) -> ResultWithDefaultError<()> {
        match api_client.get_time_entry(id).await {
            Err(_) => println!("{}", format!("No time entry found with id {id}").yellow()),
            Ok(entry) => match api_client.delete_time_entry(entry.workspace_id, id).await {
                Err(error) => println!("{}\n{}", "Couldn't delete time entry".red(), error),
                Ok(()) => println!("{}\n{}", "Time entry deleted successfully".green(), entry),
            },
        }

        Ok(())
    }
}
