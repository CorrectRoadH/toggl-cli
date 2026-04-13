use crate::api::client::ApiClient;
use crate::error::ArgumentError;
use crate::models::ResultWithDefaultError;

pub struct ArchiveProjectCommand;

impl ArchiveProjectCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        name: String,
        archived: bool,
    ) -> ResultWithDefaultError<()> {
        let project = api_client
            .get_projects_list()
            .await?
            .into_iter()
            .find(|p| p.name == name)
            .ok_or_else(|| {
                Box::new(ArgumentError::ResourceNotFound(format!(
                    "No project found with name '{name}'"
                ))) as Box<dyn std::error::Error + Send>
            })?;

        let updated = api_client
            .set_project_archived(project.workspace_id, project.id, archived)
            .await?;
        let verb = if archived { "archived" } else { "unarchived" };
        println!("Project {verb} successfully\n{updated}");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::client::MockApiClient;
    use crate::error::ApiError;
    use crate::models::Project;
    use chrono::Utc;
    use tokio_test::{assert_err, assert_ok};

    fn mock_project(active: bool) -> Project {
        Project {
            id: 10,
            name: "Proj".to_string(),
            workspace_id: 1,
            client: None,
            is_private: false,
            active,
            at: Utc::now(),
            created_at: Utc::now(),
            color: "#06aaf5".to_string(),
            billable: None,
        }
    }

    #[tokio::test]
    async fn archive_project_sends_active_false() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_projects_list()
            .returning(|| Ok(vec![mock_project(true)]));
        api_client
            .expect_set_project_archived()
            .withf(|wid, pid, archived| *wid == 1 && *pid == 10 && *archived)
            .returning(|_, _, _| Ok(mock_project(false)));

        let result = ArchiveProjectCommand::execute(api_client, "Proj".to_string(), true).await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn unarchive_project_sends_active_true() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_projects_list()
            .returning(|| Ok(vec![mock_project(false)]));
        api_client
            .expect_set_project_archived()
            .withf(|wid, pid, archived| *wid == 1 && *pid == 10 && !*archived)
            .returning(|_, _, _| Ok(mock_project(true)));

        let result = ArchiveProjectCommand::execute(api_client, "Proj".to_string(), false).await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn archive_project_handles_not_found() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_projects_list()
            .returning(|| Ok(vec![mock_project(true)]));

        let result = ArchiveProjectCommand::execute(api_client, "Missing".to_string(), true).await;
        assert_err!(result);
    }

    #[tokio::test]
    async fn archive_project_handles_api_failure() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_projects_list()
            .returning(|| Ok(vec![mock_project(true)]));
        api_client
            .expect_set_project_archived()
            .returning(|_, _, _| Err(Box::new(ApiError::Network)));

        let result = ArchiveProjectCommand::execute(api_client, "Proj".to_string(), true).await;
        assert_err!(result);
    }
}
