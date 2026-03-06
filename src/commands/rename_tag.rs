use crate::api::client::ApiClient;
use crate::error::ArgumentError;
use crate::models::ResultWithDefaultError;

pub struct RenameTagCommand;

impl RenameTagCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        old_name: String,
        new_name: String,
    ) -> ResultWithDefaultError<()> {
        let workspace_id = api_client.get_user().await?.default_workspace_id;
        let tags = api_client.get_tags(workspace_id).await?;

        let tag = tags
            .into_iter()
            .find(|t| t.name == old_name)
            .ok_or_else(|| {
                Box::new(ArgumentError::ResourceNotFound(format!(
                    "No tag found with name '{old_name}'"
                ))) as Box<dyn std::error::Error + Send>
            })?;

        let tag = api_client
            .rename_tag(workspace_id, tag.id, new_name)
            .await?;
        println!("Tag renamed successfully\n{}", tag);

        Ok(())
    }
}
