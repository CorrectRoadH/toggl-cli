use crate::api::client::ApiClient;
use crate::models::ResultWithDefaultError;
use colored::Colorize;

pub struct MeCommand;

impl MeCommand {
    pub async fn execute(api_client: impl ApiClient) -> ResultWithDefaultError<()> {
        let user = api_client.get_user().await?;

        println!("{}", "User Profile".bold().underline());
        println!(
            "  {} {}",
            "Name:".bold(),
            user.fullname.as_deref().unwrap_or("(not set)")
        );
        println!("  {} {}", "Email:".bold(), user.email);
        println!("  {} {}", "Timezone:".bold(), user.timezone);
        println!(
            "  {} {}",
            "Default Workspace ID:".bold(),
            user.default_workspace_id
        );
        if let Some(dow) = user.beginning_of_week {
            let day = match dow {
                0 => "Sunday",
                1 => "Monday",
                2 => "Tuesday",
                3 => "Wednesday",
                4 => "Thursday",
                5 => "Friday",
                6 => "Saturday",
                _ => "Unknown",
            };
            println!("  {} {}", "Week Starts On:".bold(), day);
        }
        if let Some(ref url) = user.image_url {
            println!("  {} {}", "Avatar:".bold(), url);
        }
        if let Some(ref created) = user.created_at {
            println!("  {} {}", "Created At:".bold(), created);
        }

        Ok(())
    }
}
