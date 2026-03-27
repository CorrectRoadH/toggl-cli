use crate::api::client::ApiClient;
use crate::models::ResultWithDefaultError;
use colored::Colorize;

pub struct DeleteCommand;

impl DeleteCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        id: i64,
        json: bool,
    ) -> ResultWithDefaultError<()> {
        match api_client.get_time_entry(id).await {
            Err(_) => {
                eprintln!("{}", format!("No time entry found with id {id}").yellow());
                std::process::exit(1);
            }
            Ok(entry) => match api_client.delete_time_entry(entry.workspace_id, id).await {
                Err(error) => {
                    eprintln!("{}\n{}", "Couldn't delete time entry".red(), error);
                    return Err(error);
                }
                Ok(()) => {
                    if json {
                        println!("{{\"deleted\":true,\"id\":{}}}", id);
                    } else {
                        println!("{}\n{}", "Time entry deleted successfully".green(), entry);
                    }
                }
            },
        }

        Ok(())
    }
}
