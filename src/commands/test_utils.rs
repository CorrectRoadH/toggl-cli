#[cfg(test)]
#[allow(clippy::module_inception)]
pub mod test_utils {
    use crate::models::User;

    /// Create a mock user for testing
    #[allow(dead_code)]
    pub fn mock_user() -> User {
        User {
            api_token: "token".to_string(),
            email: "test@example.com".to_string(),
            fullname: Some("Test".to_string()),
            timezone: "UTC".to_string(),
            default_workspace_id: 1,
            beginning_of_week: None,
            image_url: None,
            created_at: None,
            updated_at: None,
            country_id: None,
            has_password: None,
        }
    }

    /// Macro to reduce boilerplate in command tests
    #[macro_export]
    macro_rules! setup_mock_api_client_with_user {
        ($api_client:ident) => {
            let user = $crate::commands::common::test_utils::mock_user();
            $api_client
                .expect_get_user()
                .returning(move || Ok(user.clone()));
        };
    }
}
