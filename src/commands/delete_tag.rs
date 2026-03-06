use crate::api::client::ApiClient;
use crate::error::ArgumentError;
use crate::models::ResultWithDefaultError;

pub struct DeleteTagCommand;

impl DeleteTagCommand {
    pub async fn execute(api_client: impl ApiClient, name: String) -> ResultWithDefaultError<()> {
        let workspace_id = api_client.get_user().await?.default_workspace_id;
        let tags = api_client.get_tags(workspace_id).await?;

        let tag = tags.into_iter().find(|t| t.name == name).ok_or_else(|| {
            Box::new(ArgumentError::ResourceNotFound(format!(
                "No tag found with name '{name}'"
            ))) as Box<dyn std::error::Error + Send>
        })?;

        api_client.delete_tag(workspace_id, tag.id).await?;
        println!("Tag deleted successfully");

        Ok(())
    }
}
