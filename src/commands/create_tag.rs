use crate::api::client::ApiClient;
use crate::models::ResultWithDefaultError;
pub struct CreateTagCommand;

impl CreateTagCommand {
    pub async fn execute(api_client: impl ApiClient, name: String) -> ResultWithDefaultError<()> {
        let workspace_id = api_client.get_user().await?.default_workspace_id;
        let tag = api_client.create_tag(workspace_id, name).await?;
        println!("Tag created successfully\n{}", tag);
        Ok(())
    }
}
