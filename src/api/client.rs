use std::collections::HashMap;

use crate::credentials;
use crate::error;
use crate::models;
use crate::models::Entities;
use crate::models::Project;
use crate::models::Tag;
use crate::models::Task;
use crate::models::TimeEntry;
use crate::models::Workspace;
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use error::ApiError;
#[cfg(test)]
use mockall::automock;
use models::{ResultWithDefaultError, User};
use reqwest::Client;
use reqwest::{header, RequestBuilder};
use serde::{de, Serialize};
use serde_json::Value;

use super::models::NetworkClient;
use super::models::NetworkCreateClient;
use super::models::NetworkCreateProject;
use super::models::NetworkCreateTag;
use super::models::NetworkCreateTask;
use super::models::NetworkCreateWorkspace;
use super::models::NetworkProject;
use super::models::NetworkRenameClient;
use super::models::NetworkRenameProject;
use super::models::NetworkRenameTag;
use super::models::NetworkTag;
use super::models::NetworkTask;
use super::models::NetworkTimeEntry;
use super::models::NetworkUpdateTask;
use super::models::NetworkUpdateWorkspace;
use super::models::NetworkWorkspace;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait ApiClient {
    async fn get_user(&self) -> ResultWithDefaultError<User>;
    async fn get_entities(&self) -> ResultWithDefaultError<Entities>;

    async fn create_time_entry(&self, time_entry: TimeEntry) -> ResultWithDefaultError<i64>;
    async fn update_time_entry(&self, time_entry: TimeEntry) -> ResultWithDefaultError<i64>;

    async fn get_time_entries_filtered(
        &self,
        since: Option<String>,
        until: Option<String>,
    ) -> ResultWithDefaultError<Vec<TimeEntry>>;

    async fn delete_time_entry(
        &self,
        workspace_id: i64,
        time_entry_id: i64,
    ) -> ResultWithDefaultError<()>;

    async fn get_current_time_entry(&self) -> ResultWithDefaultError<Option<TimeEntry>>;

    async fn stop_time_entry(
        &self,
        workspace_id: i64,
        time_entry_id: i64,
    ) -> ResultWithDefaultError<TimeEntry>;

    async fn bulk_update_time_entries(
        &self,
        workspace_id: i64,
        time_entry_ids: Vec<i64>,
        patch: Value,
    ) -> ResultWithDefaultError<Value>;

    async fn create_project(
        &self,
        workspace_id: i64,
        name: String,
        color: String,
    ) -> ResultWithDefaultError<Project>;

    async fn delete_project(
        &self,
        workspace_id: i64,
        project_id: i64,
    ) -> ResultWithDefaultError<()>;

    async fn rename_project(
        &self,
        workspace_id: i64,
        project_id: i64,
        new_name: String,
    ) -> ResultWithDefaultError<Project>;

    async fn get_tags(&self, workspace_id: i64) -> ResultWithDefaultError<Vec<Tag>>;

    async fn create_tag(&self, workspace_id: i64, name: String) -> ResultWithDefaultError<Tag>;

    async fn rename_tag(
        &self,
        workspace_id: i64,
        tag_id: i64,
        new_name: String,
    ) -> ResultWithDefaultError<Tag>;

    async fn delete_tag(&self, workspace_id: i64, tag_id: i64) -> ResultWithDefaultError<()>;

    async fn get_clients(&self, workspace_id: i64) -> ResultWithDefaultError<Vec<models::Client>>;

    async fn create_client(
        &self,
        workspace_id: i64,
        name: String,
    ) -> ResultWithDefaultError<models::Client>;

    async fn rename_client(
        &self,
        workspace_id: i64,
        client_id: i64,
        new_name: String,
    ) -> ResultWithDefaultError<models::Client>;

    async fn delete_client(&self, workspace_id: i64, client_id: i64) -> ResultWithDefaultError<()>;

    async fn get_time_entry(&self, time_entry_id: i64) -> ResultWithDefaultError<TimeEntry>;

    async fn create_workspace(
        &self,
        organization_id: i64,
        name: String,
    ) -> ResultWithDefaultError<Workspace>;

    async fn rename_workspace(
        &self,
        workspace_id: i64,
        new_name: String,
    ) -> ResultWithDefaultError<Workspace>;

    async fn get_preferences(&self) -> ResultWithDefaultError<Value>;

    async fn update_preferences(&self, preferences: Value) -> ResultWithDefaultError<Value>;

    async fn create_task(
        &self,
        workspace_id: i64,
        project_id: i64,
        name: String,
        active: Option<bool>,
        estimated_seconds: Option<i64>,
        user_id: Option<i64>,
    ) -> ResultWithDefaultError<Task>;

    #[allow(clippy::too_many_arguments)]
    async fn update_task(
        &self,
        workspace_id: i64,
        project_id: i64,
        task_id: i64,
        name: Option<String>,
        active: Option<bool>,
        estimated_seconds: Option<i64>,
        user_id: Option<i64>,
    ) -> ResultWithDefaultError<Task>;

    async fn delete_task(
        &self,
        workspace_id: i64,
        project_id: i64,
        task_id: i64,
    ) -> ResultWithDefaultError<()>;
}

pub struct V9ApiClient {
    http_client: Client,
    base_url: String,
}

impl V9ApiClient {
    async fn get_time_entries(
        &self,
        since: Option<&str>,
        until: Option<&str>,
    ) -> ResultWithDefaultError<Vec<NetworkTimeEntry>> {
        let mut url = format!("{}/me/time_entries", self.base_url);
        let mut params: Vec<String> = Vec::new();
        if let Some(since) = since {
            params.push(format!("start_date={since}"));
        }
        if let Some(until) = until {
            params.push(format!("end_date={until}"));
        }
        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }
        self.get::<Vec<NetworkTimeEntry>>(url).await
    }

    async fn get_projects(&self) -> ResultWithDefaultError<Vec<NetworkProject>> {
        let url = format!("{}/me/projects", self.base_url);
        self.get::<Vec<NetworkProject>>(url).await
    }

    async fn get_clients(&self) -> ResultWithDefaultError<Vec<NetworkClient>> {
        let url = format!("{}/me/clients", self.base_url);
        self.get::<Vec<NetworkClient>>(url).await
    }

    async fn get_tasks(&self) -> ResultWithDefaultError<Vec<NetworkTask>> {
        let url = format!("{}/me/tasks", self.base_url);
        self.get::<Vec<NetworkTask>>(url).await
    }

    async fn get_workspaces(&self) -> ResultWithDefaultError<Vec<NetworkWorkspace>> {
        let url = format!("{}/me/workspaces", self.base_url);
        self.get::<Vec<NetworkWorkspace>>(url).await
    }

    async fn get_workspace_tags(
        &self,
        workspace_id: i64,
    ) -> ResultWithDefaultError<Vec<NetworkTag>> {
        let url = format!("{}/workspaces/{}/tags", self.base_url, workspace_id);
        self.get::<Vec<NetworkTag>>(url).await
    }

    fn network_project_to_project(network_project: NetworkProject) -> Project {
        Project {
            id: network_project.id,
            name: network_project.name,
            workspace_id: network_project.workspace_id,
            client: None,
            is_private: network_project.is_private,
            active: network_project.active,
            at: network_project.at,
            created_at: network_project.created_at,
            color: network_project.color,
            billable: network_project.billable,
        }
    }

    fn network_task_to_task(network_task: NetworkTask, project: Project) -> Task {
        Task {
            id: network_task.id,
            name: network_task.name,
            project,
            workspace_id: network_task.workspace_id,
        }
    }

    fn map_clients(network_clients: Vec<NetworkClient>) -> HashMap<i64, crate::models::Client> {
        network_clients
            .into_iter()
            .map(|client| {
                (
                    client.id,
                    crate::models::Client {
                        id: client.id,
                        name: client.name,
                        workspace_id: client.wid,
                    },
                )
            })
            .collect()
    }

    fn map_projects(
        network_projects: Vec<NetworkProject>,
        clients: &HashMap<i64, crate::models::Client>,
    ) -> HashMap<i64, Project> {
        network_projects
            .into_iter()
            .map(|project| {
                (
                    project.id,
                    Project {
                        id: project.id,
                        name: project.name,
                        workspace_id: project.workspace_id,
                        client: clients.get(&project.client_id.unwrap_or(-1)).cloned(),
                        is_private: project.is_private,
                        active: project.active,
                        at: project.at,
                        created_at: project.created_at,
                        color: project.color,
                        billable: project.billable,
                    },
                )
            })
            .collect()
    }

    fn map_tasks(
        network_tasks: Vec<NetworkTask>,
        projects: &HashMap<i64, Project>,
    ) -> HashMap<i64, Task> {
        network_tasks
            .into_iter()
            .filter_map(|task| {
                projects.get(&task.project_id).map(|project| {
                    (
                        task.id,
                        Task {
                            id: task.id,
                            name: task.name,
                            project: project.clone(),
                            workspace_id: task.workspace_id,
                        },
                    )
                })
            })
            .collect()
    }

    fn map_network_time_entry(
        network_entry: NetworkTimeEntry,
        projects: &HashMap<i64, Project>,
        tasks: &HashMap<i64, Task>,
    ) -> TimeEntry {
        TimeEntry {
            id: network_entry.id,
            description: network_entry.description,
            start: network_entry.start,
            stop: network_entry.stop,
            duration: network_entry.duration,
            billable: network_entry.billable,
            workspace_id: network_entry.workspace_id,
            tags: network_entry.tags.unwrap_or_default(),
            project: projects
                .get(&network_entry.project_id.unwrap_or(-1))
                .cloned(),
            task: tasks.get(&network_entry.task_id.unwrap_or(-1)).cloned(),
            ..Default::default()
        }
    }

    async fn get_project_by_id(&self, project_id: i64) -> ResultWithDefaultError<Project> {
        let network_project = self
            .get_projects()
            .await?
            .into_iter()
            .find(|project| project.id == project_id)
            .ok_or_else(|| -> Box<dyn std::error::Error + Send> {
                Box::new(ApiError::Deserialization)
            })?;
        Ok(Self::network_project_to_project(network_project))
    }

    async fn get_current_network_time_entry(
        &self,
    ) -> ResultWithDefaultError<Option<NetworkTimeEntry>> {
        let url = format!("{}/me/time_entries/current", self.base_url);
        match self.http_client.get(url).send().await {
            Err(error) => Err(Box::new(ApiError::NetworkWithMessage(error.to_string()))),
            Ok(response) => {
                if response.status() == reqwest::StatusCode::NOT_FOUND
                    || response.status() == reqwest::StatusCode::NO_CONTENT
                {
                    return Ok(None);
                }
                if !response.status().is_success() {
                    return Err(Box::new(ApiError::Deserialization));
                }
                match response.json::<NetworkTimeEntry>().await {
                    Err(_) => Err(Box::new(ApiError::Deserialization)),
                    Ok(entry) => Ok(Some(entry)),
                }
            }
        }
    }

    pub fn from_credentials(
        credentials: credentials::Credentials,
        proxy: Option<String>,
    ) -> ResultWithDefaultError<V9ApiClient> {
        let auth_string = credentials.api_token + ":api_token";
        let header_content =
            "Basic ".to_string() + general_purpose::STANDARD.encode(auth_string).as_str();
        let mut headers = header::HeaderMap::new();
        let auth_header =
            header::HeaderValue::from_str(header_content.as_str()).expect("Invalid header value");
        headers.insert(header::AUTHORIZATION, auth_header);

        let base_client = Client::builder().default_headers(headers);
        let http_client = {
            if let Some(proxy) = proxy {
                base_client.proxy(reqwest::Proxy::all(proxy).expect("Invalid proxy"))
            } else {
                base_client
            }
        }
        .build()
        .expect("Couldn't build a http client");
        let api_client = Self {
            http_client,
            base_url: "https://api.track.toggl.com/api/v9".to_string(),
        };
        Ok(api_client)
    }

    async fn get<T: de::DeserializeOwned>(&self, url: String) -> ResultWithDefaultError<T> {
        V9ApiClient::send::<T>(self.http_client.get(url)).await
    }

    async fn put<T: de::DeserializeOwned, Body: Serialize>(
        &self,
        url: String,
        body: &Body,
    ) -> ResultWithDefaultError<T> {
        V9ApiClient::send::<T>(self.http_client.put(url).json(body)).await
    }

    async fn post<T: de::DeserializeOwned, Body: Serialize>(
        &self,
        url: String,
        body: &Body,
    ) -> ResultWithDefaultError<T> {
        V9ApiClient::send::<T>(self.http_client.post(url).json(body)).await
    }

    async fn patch<T: de::DeserializeOwned, Body: Serialize>(
        &self,
        url: String,
        body: &Body,
    ) -> ResultWithDefaultError<T> {
        V9ApiClient::send::<T>(self.http_client.patch(url).json(body)).await
    }

    async fn patch_without_body<T: de::DeserializeOwned>(
        &self,
        url: String,
    ) -> ResultWithDefaultError<T> {
        V9ApiClient::send::<T>(self.http_client.patch(url)).await
    }

    async fn send<T: de::DeserializeOwned>(request: RequestBuilder) -> ResultWithDefaultError<T> {
        match request.send().await {
            Err(error) => Err(Box::new(ApiError::NetworkWithMessage(error.to_string()))),
            Ok(response) => {
                let status = response.status();
                let body = response.text().await.map_err(|error| {
                    Box::new(ApiError::NetworkWithMessage(error.to_string()))
                        as Box<dyn std::error::Error + Send>
                })?;

                if !status.is_success() {
                    return Err(Box::new(ApiError::NetworkWithMessage(format!(
                        "HTTP {} {}",
                        status.as_u16(),
                        summarize_response_body(&body)
                    ))));
                }

                serde_json::from_str::<T>(&body).map_err(|error| {
                    Box::new(ApiError::DeserializationWithMessage(format!(
                        "{}; response body: {}",
                        error,
                        summarize_response_body(&body)
                    ))) as Box<dyn std::error::Error + Send>
                })
            }
        }
    }

    async fn delete(&self, url: String) -> ResultWithDefaultError<()> {
        match self.http_client.delete(url).send().await {
            Err(error) => Err(Box::new(ApiError::NetworkWithMessage(error.to_string()))),
            Ok(response) => {
                if response.status().is_success() {
                    Ok(())
                } else {
                    Err(Box::new(ApiError::Deserialization))
                }
            }
        }
    }
}

fn summarize_response_body(body: &str) -> String {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return "(empty response body)".to_string();
    }

    const MAX_LEN: usize = 400;
    let summary: String = trimmed.chars().take(MAX_LEN).collect();
    if trimmed.chars().count() > MAX_LEN {
        format!("{summary}...")
    } else {
        summary
    }
}

#[async_trait]
impl ApiClient for V9ApiClient {
    async fn get_user(&self) -> ResultWithDefaultError<User> {
        let url = format!("{}/me", self.base_url);
        return self.get::<User>(url).await;
    }

    async fn create_time_entry(&self, time_entry: TimeEntry) -> ResultWithDefaultError<i64> {
        let url = format!(
            "{}/workspaces/{}/time_entries",
            self.base_url, time_entry.workspace_id
        );
        let network_time_entry = self
            .post::<NetworkTimeEntry, NetworkTimeEntry>(url, &time_entry.into())
            .await?;
        return Ok(network_time_entry.id);
    }

    async fn update_time_entry(&self, time_entry: TimeEntry) -> ResultWithDefaultError<i64> {
        let url = format!(
            "{}/workspaces/{}/time_entries/{}",
            self.base_url, time_entry.workspace_id, time_entry.id
        );
        let network_time_entry = self
            .put::<NetworkTimeEntry, NetworkTimeEntry>(url, &time_entry.into())
            .await?;
        return Ok(network_time_entry.id);
    }

    async fn delete_time_entry(
        &self,
        workspace_id: i64,
        time_entry_id: i64,
    ) -> ResultWithDefaultError<()> {
        let url = format!(
            "{}/workspaces/{}/time_entries/{}",
            self.base_url, workspace_id, time_entry_id
        );
        self.delete(url).await
    }

    async fn get_current_time_entry(&self) -> ResultWithDefaultError<Option<TimeEntry>> {
        let (network_entry, network_projects, network_tasks, network_clients) = tokio::join!(
            self.get_current_network_time_entry(),
            self.get_projects(),
            self.get_tasks(),
            self.get_clients(),
        );

        let network_entry = match network_entry? {
            Some(entry) => entry,
            None => return Ok(None),
        };

        let clients = Self::map_clients(network_clients.unwrap_or_default());
        let projects = Self::map_projects(network_projects.unwrap_or_default(), &clients);
        let tasks = Self::map_tasks(network_tasks.unwrap_or_default(), &projects);

        Ok(Some(Self::map_network_time_entry(
            network_entry,
            &projects,
            &tasks,
        )))
    }

    async fn stop_time_entry(
        &self,
        workspace_id: i64,
        time_entry_id: i64,
    ) -> ResultWithDefaultError<TimeEntry> {
        let url = format!(
            "{}/workspaces/{}/time_entries/{}/stop",
            self.base_url, workspace_id, time_entry_id
        );
        let network_entry = self.patch_without_body::<NetworkTimeEntry>(url).await?;
        Ok(TimeEntry {
            id: network_entry.id,
            description: network_entry.description,
            start: network_entry.start,
            stop: network_entry.stop,
            duration: network_entry.duration,
            billable: network_entry.billable,
            workspace_id: network_entry.workspace_id,
            tags: network_entry.tags.unwrap_or_default(),
            project: None,
            task: None,
            ..Default::default()
        })
    }

    async fn bulk_update_time_entries(
        &self,
        workspace_id: i64,
        time_entry_ids: Vec<i64>,
        patch: Value,
    ) -> ResultWithDefaultError<Value> {
        let ids = time_entry_ids
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let url = format!(
            "{}/workspaces/{}/time_entries/{}",
            self.base_url, workspace_id, ids
        );
        self.patch::<Value, Value>(url, &patch).await
    }

    async fn get_time_entries_filtered(
        &self,
        since: Option<String>,
        until: Option<String>,
    ) -> ResultWithDefaultError<Vec<TimeEntry>> {
        let (network_entries, network_projects, network_tasks, network_clients) = tokio::join!(
            self.get_time_entries(since.as_deref(), until.as_deref()),
            self.get_projects(),
            self.get_tasks(),
            self.get_clients(),
        );

        let clients: HashMap<i64, crate::models::Client> = network_clients
            .unwrap_or_default()
            .into_iter()
            .map(|c| {
                (
                    c.id,
                    crate::models::Client {
                        id: c.id,
                        name: c.name,
                        workspace_id: c.wid,
                    },
                )
            })
            .collect();

        let projects: HashMap<i64, Project> = network_projects
            .unwrap_or_default()
            .into_iter()
            .map(|p| {
                (
                    p.id,
                    Project {
                        id: p.id,
                        name: p.name.clone(),
                        workspace_id: p.workspace_id,
                        client: clients.get(&p.client_id.unwrap_or(-1)).cloned(),
                        is_private: p.is_private,
                        active: p.active,
                        at: p.at,
                        created_at: p.created_at,
                        color: p.color,
                        billable: p.billable,
                    },
                )
            })
            .collect();

        let tasks: HashMap<i64, Task> = network_tasks
            .unwrap_or_default()
            .into_iter()
            .filter_map(|t| {
                projects.get(&t.project_id).map(|project| {
                    (
                        t.id,
                        Task {
                            id: t.id,
                            name: t.name,
                            project: project.clone(),
                            workspace_id: t.workspace_id,
                        },
                    )
                })
            })
            .collect();

        let entries = network_entries
            .unwrap_or_default()
            .into_iter()
            .map(|te| TimeEntry {
                id: te.id,
                description: te.description,
                start: te.start,
                stop: te.stop,
                duration: te.duration,
                billable: te.billable,
                workspace_id: te.workspace_id,
                tags: te.tags.unwrap_or_default(),
                project: projects.get(&te.project_id.unwrap_or(-1)).cloned(),
                task: tasks.get(&te.task_id.unwrap_or(-1)).cloned(),
                ..Default::default()
            })
            .collect();

        Ok(entries)
    }

    async fn create_project(
        &self,
        workspace_id: i64,
        name: String,
        color: String,
    ) -> ResultWithDefaultError<Project> {
        let url = format!("{}/workspaces/{}/projects", self.base_url, workspace_id);
        let body = NetworkCreateProject {
            name,
            workspace_id,
            color,
            is_private: false,
            active: true,
        };
        let network_project = self
            .post::<NetworkProject, NetworkCreateProject>(url, &body)
            .await?;
        Ok(Project {
            id: network_project.id,
            name: network_project.name,
            workspace_id: network_project.workspace_id,
            client: None,
            is_private: network_project.is_private,
            active: network_project.active,
            at: network_project.at,
            created_at: network_project.created_at,
            color: network_project.color,
            billable: network_project.billable,
        })
    }

    async fn delete_project(
        &self,
        workspace_id: i64,
        project_id: i64,
    ) -> ResultWithDefaultError<()> {
        let url = format!(
            "{}/workspaces/{}/projects/{}",
            self.base_url, workspace_id, project_id
        );
        self.delete(url).await
    }

    async fn rename_project(
        &self,
        workspace_id: i64,
        project_id: i64,
        new_name: String,
    ) -> ResultWithDefaultError<Project> {
        let url = format!(
            "{}/workspaces/{}/projects/{}",
            self.base_url, workspace_id, project_id
        );
        let body = NetworkRenameProject { name: new_name };
        let network_project = self
            .put::<NetworkProject, NetworkRenameProject>(url, &body)
            .await?;
        Ok(Project {
            id: network_project.id,
            name: network_project.name,
            workspace_id: network_project.workspace_id,
            client: None,
            is_private: network_project.is_private,
            active: network_project.active,
            at: network_project.at,
            created_at: network_project.created_at,
            color: network_project.color,
            billable: network_project.billable,
        })
    }

    async fn get_tags(&self, workspace_id: i64) -> ResultWithDefaultError<Vec<Tag>> {
        let network_tags = self.get_workspace_tags(workspace_id).await?;
        Ok(network_tags
            .into_iter()
            .map(|t| Tag {
                id: t.id,
                name: t.name,
                workspace_id: t.workspace_id,
            })
            .collect())
    }

    async fn create_tag(&self, workspace_id: i64, name: String) -> ResultWithDefaultError<Tag> {
        let url = format!("{}/workspaces/{}/tags", self.base_url, workspace_id);
        let body = NetworkCreateTag { name, workspace_id };
        let network_tag = self
            .post::<NetworkTag, NetworkCreateTag>(url, &body)
            .await?;
        Ok(Tag {
            id: network_tag.id,
            name: network_tag.name,
            workspace_id: network_tag.workspace_id,
        })
    }

    async fn rename_tag(
        &self,
        workspace_id: i64,
        tag_id: i64,
        new_name: String,
    ) -> ResultWithDefaultError<Tag> {
        let url = format!(
            "{}/workspaces/{}/tags/{}",
            self.base_url, workspace_id, tag_id
        );
        let body = NetworkRenameTag {
            name: new_name,
            workspace_id,
        };
        let network_tag = self.put::<NetworkTag, NetworkRenameTag>(url, &body).await?;
        Ok(Tag {
            id: network_tag.id,
            name: network_tag.name,
            workspace_id: network_tag.workspace_id,
        })
    }

    async fn delete_tag(&self, workspace_id: i64, tag_id: i64) -> ResultWithDefaultError<()> {
        let url = format!(
            "{}/workspaces/{}/tags/{}",
            self.base_url, workspace_id, tag_id
        );
        self.delete(url).await
    }

    async fn get_clients(&self, workspace_id: i64) -> ResultWithDefaultError<Vec<models::Client>> {
        let url = format!("{}/workspaces/{}/clients", self.base_url, workspace_id);
        let network_clients = self.get::<Vec<NetworkClient>>(url).await?;
        Ok(network_clients
            .into_iter()
            .map(|c| models::Client {
                id: c.id,
                name: c.name,
                workspace_id: c.wid,
            })
            .collect())
    }

    async fn create_client(
        &self,
        workspace_id: i64,
        name: String,
    ) -> ResultWithDefaultError<models::Client> {
        let url = format!("{}/workspaces/{}/clients", self.base_url, workspace_id);
        let body = NetworkCreateClient {
            name,
            wid: workspace_id,
        };
        let network_client = self
            .post::<NetworkClient, NetworkCreateClient>(url, &body)
            .await?;
        Ok(models::Client {
            id: network_client.id,
            name: network_client.name,
            workspace_id: network_client.wid,
        })
    }

    async fn rename_client(
        &self,
        workspace_id: i64,
        client_id: i64,
        new_name: String,
    ) -> ResultWithDefaultError<models::Client> {
        let url = format!(
            "{}/workspaces/{}/clients/{}",
            self.base_url, workspace_id, client_id
        );
        let body = NetworkRenameClient {
            name: new_name,
            wid: workspace_id,
        };
        let network_client = self
            .put::<NetworkClient, NetworkRenameClient>(url, &body)
            .await?;
        Ok(models::Client {
            id: network_client.id,
            name: network_client.name,
            workspace_id: network_client.wid,
        })
    }

    async fn delete_client(&self, workspace_id: i64, client_id: i64) -> ResultWithDefaultError<()> {
        let url = format!(
            "{}/workspaces/{}/clients/{}",
            self.base_url, workspace_id, client_id
        );
        self.delete(url).await
    }

    async fn get_time_entry(&self, time_entry_id: i64) -> ResultWithDefaultError<TimeEntry> {
        let url = format!("{}/me/time_entries/{}", self.base_url, time_entry_id);
        let network_entry = self.get::<NetworkTimeEntry>(url).await?;
        Ok(TimeEntry {
            id: network_entry.id,
            description: network_entry.description,
            start: network_entry.start,
            stop: network_entry.stop,
            duration: network_entry.duration,
            billable: network_entry.billable,
            workspace_id: network_entry.workspace_id,
            tags: network_entry.tags.unwrap_or_default(),
            project: None,
            task: None,
            ..Default::default()
        })
    }

    async fn create_workspace(
        &self,
        organization_id: i64,
        name: String,
    ) -> ResultWithDefaultError<Workspace> {
        let url = format!(
            "{}/organizations/{}/workspaces",
            self.base_url, organization_id
        );
        let body = NetworkCreateWorkspace { name };
        let network_workspace = self
            .post::<NetworkWorkspace, NetworkCreateWorkspace>(url, &body)
            .await?;
        Ok(Workspace {
            id: network_workspace.id,
            name: network_workspace.name,
            admin: network_workspace.admin,
        })
    }

    async fn rename_workspace(
        &self,
        workspace_id: i64,
        new_name: String,
    ) -> ResultWithDefaultError<Workspace> {
        let url = format!("{}/workspaces/{}", self.base_url, workspace_id);
        let body = NetworkUpdateWorkspace {
            name: Some(new_name),
        };
        let network_workspace = self
            .put::<NetworkWorkspace, NetworkUpdateWorkspace>(url, &body)
            .await?;
        Ok(Workspace {
            id: network_workspace.id,
            name: network_workspace.name,
            admin: network_workspace.admin,
        })
    }

    async fn get_preferences(&self) -> ResultWithDefaultError<Value> {
        let url = format!("{}/me/preferences", self.base_url);
        self.get::<Value>(url).await
    }

    async fn update_preferences(&self, preferences: Value) -> ResultWithDefaultError<Value> {
        let url = format!("{}/me/preferences", self.base_url);
        self.post::<Value, Value>(url, &preferences).await
    }

    async fn create_task(
        &self,
        workspace_id: i64,
        project_id: i64,
        name: String,
        active: Option<bool>,
        estimated_seconds: Option<i64>,
        user_id: Option<i64>,
    ) -> ResultWithDefaultError<Task> {
        let url = format!(
            "{}/workspaces/{}/projects/{}/tasks",
            self.base_url, workspace_id, project_id
        );
        let body = NetworkCreateTask {
            name,
            active,
            estimated_seconds,
            user_id,
        };
        let network_task = self
            .post::<NetworkTask, NetworkCreateTask>(url, &body)
            .await?;
        let project = self.get_project_by_id(project_id).await?;
        Ok(Self::network_task_to_task(network_task, project))
    }

    #[allow(clippy::too_many_arguments)]
    async fn update_task(
        &self,
        workspace_id: i64,
        project_id: i64,
        task_id: i64,
        name: Option<String>,
        active: Option<bool>,
        estimated_seconds: Option<i64>,
        user_id: Option<i64>,
    ) -> ResultWithDefaultError<Task> {
        let url = format!(
            "{}/workspaces/{}/projects/{}/tasks/{}",
            self.base_url, workspace_id, project_id, task_id
        );
        let body = NetworkUpdateTask {
            name,
            active,
            estimated_seconds,
            user_id,
        };
        let network_task = self
            .put::<NetworkTask, NetworkUpdateTask>(url, &body)
            .await?;
        let project = self.get_project_by_id(project_id).await?;
        Ok(Self::network_task_to_task(network_task, project))
    }

    async fn delete_task(
        &self,
        workspace_id: i64,
        project_id: i64,
        task_id: i64,
    ) -> ResultWithDefaultError<()> {
        let url = format!(
            "{}/workspaces/{}/projects/{}/tasks/{}",
            self.base_url, workspace_id, project_id, task_id
        );
        self.delete(url).await
    }

    async fn get_entities(&self) -> ResultWithDefaultError<Entities> {
        let (
            network_time_entries,
            network_projects,
            network_tasks,
            network_clients,
            network_workspaces,
        ) = tokio::join!(
            self.get_time_entries(None, None),
            self.get_projects(),
            self.get_tasks(),
            self.get_clients(),
            self.get_workspaces(),
        );

        let clients: HashMap<i64, crate::models::Client> = network_clients
            .unwrap_or_default()
            .iter()
            .map(|c| {
                (
                    c.id,
                    crate::models::Client {
                        id: c.id,
                        name: c.name.clone(),
                        workspace_id: c.wid,
                    },
                )
            })
            .collect();

        let projects: HashMap<i64, Project> = network_projects
            .unwrap_or_default()
            .iter()
            .map(|p| {
                (
                    p.id,
                    Project {
                        id: p.id,
                        name: p.name.clone(),
                        workspace_id: p.workspace_id,
                        client: clients.get(&p.client_id.unwrap_or(-1)).cloned(),
                        is_private: p.is_private,
                        active: p.active,
                        at: p.at,
                        created_at: p.created_at,
                        color: p.color.clone(),
                        billable: p.billable,
                    },
                )
            })
            .collect();

        let tasks: HashMap<i64, Task> = network_tasks
            .unwrap_or_default()
            .iter()
            .map(|t| {
                (
                    t.id,
                    Task {
                        id: t.id,
                        name: t.name.clone(),
                        project: projects.get(&t.project_id).unwrap().clone(),
                        workspace_id: t.workspace_id,
                    },
                )
            })
            .collect();

        let time_entries = network_time_entries
            .unwrap_or_default()
            .iter()
            .map(|te| TimeEntry {
                id: te.id,
                description: te.description.clone(),
                start: te.start,
                stop: te.stop,
                duration: te.duration,
                billable: te.billable,
                workspace_id: te.workspace_id,
                tags: te.tags.clone().unwrap_or_default(),
                project: projects.get(&te.project_id.unwrap_or(-1)).cloned(),
                task: tasks.get(&te.task_id.unwrap_or(-1)).cloned(),
                ..Default::default()
            })
            .collect();

        let workspaces = network_workspaces
            .unwrap_or_default()
            .iter()
            .map(|w| Workspace {
                id: w.id,
                name: w.name.clone(),
                admin: w.admin,
            })
            .collect();

        Ok(Entities {
            time_entries,
            projects,
            tasks,
            clients,
            workspaces,
            tags: Vec::new(),
        })
    }
}
