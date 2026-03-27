use crate::api::client::ApiClient;
use crate::models::ResultWithDefaultError;
use colored::Colorize;
use std::io::{self, BufWriter, Write};

pub enum OrganizationAction {
    List { json: bool },
    Show { id: i64, json: bool },
}

pub struct OrganizationCommand;

impl OrganizationCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        action: OrganizationAction,
    ) -> ResultWithDefaultError<()> {
        let stdout = io::stdout();
        let mut handle = BufWriter::new(stdout);

        match action {
            OrganizationAction::List { json } => {
                let organizations = api_client.get_organizations().await?;
                if json {
                    let json_string = serde_json::to_string(&organizations)
                        .expect("failed to serialize organizations to JSON");
                    writeln!(handle, "{json_string}").expect("failed to print");
                } else if organizations.is_empty() {
                    writeln!(handle, "{}", "No organizations found".yellow())
                        .expect("failed to print");
                } else {
                    for organization in organizations {
                        writeln!(handle, "{organization}").expect("failed to print");
                    }
                }
            }
            OrganizationAction::Show { id, json } => {
                let organization = api_client.get_organization(id).await?;
                if json {
                    let json_string = serde_json::to_string(&organization)
                        .expect("failed to serialize organization to JSON");
                    writeln!(handle, "{json_string}").expect("failed to print");
                } else {
                    writeln!(handle, "{}", "Organization Details".bold().underline())
                        .expect("failed to print");
                    writeln!(handle, "  {} {}", "ID:".bold(), organization.id)
                        .expect("failed to print");
                    writeln!(handle, "  {} {}", "Name:".bold(), organization.name)
                        .expect("failed to print");
                    writeln!(
                        handle,
                        "  {} {}",
                        "Role:".bold(),
                        if organization.admin {
                            "Admin"
                        } else {
                            "Member"
                        }
                    )
                    .expect("failed to print");
                    if let Some(workspace_id) = organization.workspace_id {
                        writeln!(handle, "  {} {}", "Workspace ID:".bold(), workspace_id)
                            .expect("failed to print");
                    }
                    if let Some(workspace_name) = organization.workspace_name {
                        writeln!(handle, "  {} {}", "Workspace:".bold(), workspace_name)
                            .expect("failed to print");
                    }
                    if let Some(plan) = organization.pricing_plan_name {
                        writeln!(handle, "  {} {}", "Plan:".bold(), plan).expect("failed to print");
                    }
                    if !organization.permissions.is_empty() {
                        writeln!(
                            handle,
                            "  {} {}",
                            "Permissions:".bold(),
                            organization.permissions.join(", ")
                        )
                        .expect("failed to print");
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::client::MockApiClient;
    use crate::error::ApiError;
    use crate::models::Organization;
    use tokio_test::{assert_err, assert_ok};

    fn mock_organization() -> Organization {
        Organization {
            id: 42,
            name: "Platform Org".to_string(),
            admin: true,
            workspace_id: Some(7),
            workspace_name: Some("Platform Workspace".to_string()),
            pricing_plan_name: Some("Premium".to_string()),
            permissions: vec!["workspace:create".to_string()],
        }
    }

    #[tokio::test]
    async fn organization_list_returns_ok_on_success() {
        let mut api_client = MockApiClient::new();
        let organization = mock_organization();
        api_client
            .expect_get_organizations()
            .returning(move || Ok(vec![organization.clone()]));

        let result =
            OrganizationCommand::execute(api_client, OrganizationAction::List { json: true }).await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn organization_list_returns_error_on_api_failure() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_organizations()
            .returning(|| Err(Box::new(ApiError::Network)));

        let result =
            OrganizationCommand::execute(api_client, OrganizationAction::List { json: false })
                .await;
        assert_err!(result);
    }

    #[tokio::test]
    async fn organization_show_returns_ok_on_success() {
        let mut api_client = MockApiClient::new();
        let organization = mock_organization();
        api_client
            .expect_get_organization()
            .withf(|id| *id == 42)
            .returning(move |_| Ok(organization.clone()));

        let result = OrganizationCommand::execute(
            api_client,
            OrganizationAction::Show {
                id: 42,
                json: false,
            },
        )
        .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn organization_show_returns_error_on_api_failure() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_organization()
            .returning(|_| Err(Box::new(ApiError::Network)));

        let result = OrganizationCommand::execute(
            api_client,
            OrganizationAction::Show { id: 42, json: true },
        )
        .await;
        assert_err!(result);
    }
}
