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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::client::MockApiClient;
    use crate::error::ApiError;
    use crate::models::User;
    use tokio_test::{assert_err, assert_ok};

    fn mock_user() -> User {
        User {
            api_token: "token".to_string(),
            email: "test@example.com".to_string(),
            fullname: Some("Test User".to_string()),
            timezone: "UTC".to_string(),
            default_workspace_id: 1,
            beginning_of_week: Some(1),
            image_url: Some("https://example.com/avatar.png".to_string()),
            created_at: Some("2024-01-01T00:00:00Z".to_string()),
            updated_at: None,
            country_id: None,
            has_password: None,
        }
    }

    #[tokio::test]
    async fn me_returns_ok_on_success() {
        let mut api_client = MockApiClient::new();
        let user = mock_user();
        api_client
            .expect_get_user()
            .returning(move || Ok(user.clone()));

        let result = MeCommand::execute(api_client).await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn me_returns_error_on_api_failure() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_user()
            .returning(|| Err(Box::new(ApiError::Network)));

        let result = MeCommand::execute(api_client).await;
        assert_err!(result);
    }

    #[tokio::test]
    async fn me_handles_missing_fullname() {
        let mut api_client = MockApiClient::new();
        let mut user = mock_user();
        user.fullname = None;
        api_client
            .expect_get_user()
            .returning(move || Ok(user.clone()));

        let result = MeCommand::execute(api_client).await;
        assert_ok!(result);
    }
}
