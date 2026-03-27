use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use crate::constants;
use crate::credentials;
use crate::error;
use crate::models;
use crate::models::Entities;
use crate::models::Organization;
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
use serde::Deserialize;
use serde::{de, Serialize};
use serde_json::Value;

use super::models::NetworkClient;
use super::models::NetworkCreateClient;
use super::models::NetworkCreateProject;
use super::models::NetworkCreateTag;
use super::models::NetworkCreateTask;
use super::models::NetworkCreateWorkspace;
use super::models::NetworkOrganization;
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
    async fn get_projects_list(&self) -> ResultWithDefaultError<Vec<Project>>;
    async fn get_tasks_list(&self) -> ResultWithDefaultError<Vec<Task>>;
    async fn get_workspaces_list(&self) -> ResultWithDefaultError<Vec<Workspace>>;

    async fn create_time_entry(&self, time_entry: TimeEntry) -> ResultWithDefaultError<i64>;
    async fn update_time_entry(&self, time_entry: TimeEntry) -> ResultWithDefaultError<i64>;

    async fn get_time_entries_filtered(
        &self,
        since: Option<String>,
        until: Option<String>,
    ) -> ResultWithDefaultError<Vec<TimeEntry>>;

    async fn get_time_entries_filtered_minimal(
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
    async fn get_current_time_entry_minimal(&self) -> ResultWithDefaultError<Option<TimeEntry>>;

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

    async fn get_organizations(&self) -> ResultWithDefaultError<Vec<Organization>>;

    async fn get_organization(&self, organization_id: i64) -> ResultWithDefaultError<Organization>;

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

    async fn get_summary_report(
        &self,
        workspace_id: i64,
        body: Value,
    ) -> ResultWithDefaultError<Value>;

    async fn get_detailed_report(
        &self,
        workspace_id: i64,
        body: Value,
    ) -> ResultWithDefaultError<Value>;

    async fn get_weekly_report(
        &self,
        workspace_id: i64,
        body: Value,
    ) -> ResultWithDefaultError<Value>;
}

pub struct V9ApiClient {
    http_client: Client,
    base_url: String,
    is_official_service: bool,
    cache_namespace: String,
    last_time_entry_mutation: std::sync::Arc<std::sync::Mutex<Option<i64>>>,
    last_related_mutation: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, i64>>>,
}

#[derive(Serialize, Deserialize)]
struct CachedResponse {
    fetched_at_epoch_seconds: i64,
    #[serde(default)]
    url: Option<String>,
    body: String,
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

    fn network_organization_to_organization(
        network_organization: NetworkOrganization,
    ) -> Organization {
        Organization {
            id: network_organization.id,
            name: network_organization.name,
            admin: network_organization.admin,
            workspace_id: network_organization.workspace_id,
            workspace_name: network_organization.workspace_name,
            pricing_plan_name: network_organization.pricing_plan_name,
            permissions: network_organization.permissions.unwrap_or_default(),
        }
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
        if let Some(cached_body) = self.read_cached_body(&url) {
            return deserialize_optional_response_body(&cached_body);
        }

        match self.http_client.get(&url).send().await {
            Err(error) => Err(Box::new(ApiError::NetworkWithMessage(error.to_string()))),
            Ok(response) => {
                let status = response.status();
                if status == reqwest::StatusCode::NOT_FOUND
                    || status == reqwest::StatusCode::NO_CONTENT
                {
                    return Ok(None);
                }
                let body = response.text().await.map_err(|error| {
                    Box::new(ApiError::NetworkWithMessage(error.to_string()))
                        as Box<dyn std::error::Error + Send>
                })?;
                if !status.is_success() {
                    return Err(Box::new(api_error_for_http_status(
                        status,
                        &body,
                        self.is_official_service,
                    )));
                }

                self.write_cached_body(&url, &body);
                deserialize_optional_response_body(&body)
            }
        }
    }

    pub fn from_credentials(
        credentials: credentials::Credentials,
        proxy: Option<String>,
    ) -> ResultWithDefaultError<V9ApiClient> {
        let cache_namespace = cache_namespace_for_token(&credentials.api_token);
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
        let is_official_service = credentials.api_url.is_none();
        let base_url = credentials
            .api_url
            .unwrap_or_else(|| constants::TOGGL_API_URL_OFFICIAL.to_string());
        let api_client = Self {
            http_client,
            base_url,
            is_official_service,
            cache_namespace,
            last_time_entry_mutation: std::sync::Arc::new(std::sync::Mutex::new(None)),
            last_related_mutation: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
        };
        Ok(api_client)
    }

    async fn get<T: de::DeserializeOwned>(&self, url: String) -> ResultWithDefaultError<T> {
        if let Some(cached_body) = self.read_cached_body(&url) {
            return deserialize_response_body(&cached_body);
        }

        let response = self.http_client.get(url.clone()).send().await;
        let result = match response {
            Err(error) => Err(Box::new(ApiError::NetworkWithMessage(error.to_string()))
                as Box<dyn std::error::Error + Send>),
            Ok(response) => {
                let status = response.status();
                let body = response.text().await.map_err(|error| {
                    Box::new(ApiError::NetworkWithMessage(error.to_string()))
                        as Box<dyn std::error::Error + Send>
                })?;

                if !status.is_success() {
                    Err(Box::new(api_error_for_http_status(
                        status,
                        &body,
                        self.is_official_service,
                    )) as Box<dyn std::error::Error + Send>)
                } else {
                    self.write_cached_body(&url, &body);
                    deserialize_response_body(&body)
                }
            }
        };

        result
    }

    fn read_cached_body(&self, url: &str) -> Option<String> {
        if !is_cacheable_get_url(url) || is_http_cache_disabled() {
            return None;
        }

        // Skip cache for time entry requests if there was a recent mutation
        if url.contains("/me/time_entries") {
            if let Ok(last_mutation) = self.last_time_entry_mutation.lock() {
                if let Some(last_mutation_time) = *last_mutation {
                    let now = chrono::Utc::now().timestamp();
                    // Skip cache for 10 seconds after a time entry mutation
                    // Only for time entry related endpoints, not all /me endpoints
                    if now - last_mutation_time < 10 {
                        return None;
                    }
                }
            }
        }

        // Skip cache for related endpoints if there was a recent mutation affecting them
        if let Ok(related_mutations) = self.last_related_mutation.lock() {
            let now = chrono::Utc::now().timestamp();
            for (endpoint_pattern, last_mutation_time) in related_mutations.iter() {
                if url.contains(endpoint_pattern) && now - last_mutation_time < 10 {
                    return None;
                }
            }
        }

        let cache_path = self.cache_file_path(url)?;
        let contents = fs::read_to_string(cache_path).ok()?;
        let cached = serde_json::from_str::<CachedResponse>(&contents).ok()?;
        let age = chrono::Utc::now().timestamp() - cached.fetched_at_epoch_seconds;
        if age < 0 || age > cache_ttl_seconds_for_url(url) {
            return None;
        }
        Some(cached.body)
    }

    fn write_cached_body(&self, url: &str, body: &str) {
        if !is_cacheable_get_url(url) || is_http_cache_disabled() {
            return;
        }

        let Some(cache_path) = self.cache_file_path(url) else {
            return;
        };
        let Some(cache_dir) = cache_path.parent() else {
            return;
        };
        if fs::create_dir_all(cache_dir).is_err() {
            return;
        }

        let cached = CachedResponse {
            fetched_at_epoch_seconds: chrono::Utc::now().timestamp(),
            url: Some(url.to_string()),
            body: body.to_string(),
        };
        let Ok(serialized) = serde_json::to_string(&cached) else {
            return;
        };
        let _ = fs::write(cache_path, serialized);
    }

    fn cache_file_path(&self, url: &str) -> Option<PathBuf> {
        cache_root_dir().map(|root| {
            root.join(&self.cache_namespace)
                .join(format!("{}.json", stable_hash(url)))
        })
    }

    fn invalidate_cached_url(&self, url: &str) {
        let Some(cache_path) = self.cache_file_path(url) else {
            return;
        };
        let _ = fs::remove_file(cache_path);
    }

    fn invalidate_cached_urls(&self, urls: &[String]) {
        for url in urls {
            self.invalidate_cached_url(url);
        }
    }

    fn invalidate_cached_urls_matching<F>(&self, mut predicate: F)
    where
        F: FnMut(&str) -> bool,
    {
        let Some(dir) = cache_root_dir().map(|root| root.join(&self.cache_namespace)) else {
            return;
        };
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(contents) = fs::read_to_string(&path) else {
                continue;
            };
            let Ok(cached) = serde_json::from_str::<CachedResponse>(&contents) else {
                continue;
            };
            let Some(cached_url) = cached.url.as_deref() else {
                continue;
            };
            if predicate(cached_url) {
                let _ = fs::remove_file(path);
            }
        }
    }

    fn invalidate_caches_for_mutation(&self, mutation_url: &str) {
        let invalidation = cache_invalidation_for_mutation(&self.base_url, mutation_url);
        self.invalidate_cached_urls(&invalidation.exact_urls);
        if invalidation.invalidate_time_entries {
            self.invalidate_cached_urls_matching(|url| url.contains("/me/time_entries"));
            // Record the time of this time entry mutation
            if let Ok(mut last_mutation) = self.last_time_entry_mutation.lock() {
                *last_mutation = Some(chrono::Utc::now().timestamp());
            }
        }

        // Record mutations for related endpoints that should bypass cache temporarily
        if !invalidation.bypass_related_endpoints.is_empty() {
            if let Ok(mut related_mutations) = self.last_related_mutation.lock() {
                let now = chrono::Utc::now().timestamp();
                for endpoint in &invalidation.bypass_related_endpoints {
                    related_mutations.insert(endpoint.clone(), now);
                }
            }
        }
    }

    async fn put<T: de::DeserializeOwned, Body: Serialize>(
        &self,
        url: String,
        body: &Body,
    ) -> ResultWithDefaultError<T> {
        let result = V9ApiClient::send::<T>(
            self.http_client.put(&url).json(body),
            self.is_official_service,
        )
        .await;
        if result.is_ok() {
            self.invalidate_caches_for_mutation(&url);
        }
        result
    }

    async fn post<T: de::DeserializeOwned, Body: Serialize>(
        &self,
        url: String,
        body: &Body,
    ) -> ResultWithDefaultError<T> {
        let result = V9ApiClient::send::<T>(
            self.http_client.post(&url).json(body),
            self.is_official_service,
        )
        .await;
        if result.is_ok() {
            self.invalidate_caches_for_mutation(&url);
        }
        result
    }

    /// Parse a JSON response body as either a single `T` or the first element of `Vec<T>`.
    /// OpenToggl sometimes wraps responses in an array where official Toggl returns an object.
    fn parse_single_or_array<T: de::DeserializeOwned>(body: &str) -> ResultWithDefaultError<T> {
        serde_json::from_str::<T>(body)
            .or_else(|_| {
                serde_json::from_str::<Vec<T>>(body).and_then(|items| {
                    items.into_iter().next().ok_or_else(|| {
                        <serde_json::Error as serde::de::Error>::custom("API returned empty array")
                    })
                })
            })
            .map_err(|error| {
                Box::new(ApiError::NetworkWithMessage(format!(
                    "{error}; response body: {body}"
                ))) as Box<dyn std::error::Error + Send>
            })
    }

    fn reports_base_url(&self) -> String {
        self.base_url.replace("/api/v9", "/reports/api/v3")
    }

    async fn send_raw(
        &self,
        request: reqwest::RequestBuilder,
        url: &str,
    ) -> ResultWithDefaultError<String> {
        let response = request.send().await.map_err(|e| {
            Box::new(ApiError::NetworkWithMessage(e.to_string()))
                as Box<dyn std::error::Error + Send>
        })?;
        let status = response.status();
        let text = response.text().await.map_err(|e| {
            Box::new(ApiError::NetworkWithMessage(e.to_string()))
                as Box<dyn std::error::Error + Send>
        })?;
        if !status.is_success() {
            return Err(Box::new(api_error_for_http_status(
                status,
                &text,
                self.is_official_service,
            )));
        }
        self.invalidate_caches_for_mutation(url);
        Ok(text)
    }

    async fn patch<T: de::DeserializeOwned, Body: Serialize>(
        &self,
        url: String,
        body: &Body,
    ) -> ResultWithDefaultError<T> {
        let result = V9ApiClient::send::<T>(
            self.http_client.patch(&url).json(body),
            self.is_official_service,
        )
        .await;
        if result.is_ok() {
            self.invalidate_caches_for_mutation(&url);
        }
        result
    }

    async fn patch_without_body<T: de::DeserializeOwned>(
        &self,
        url: String,
    ) -> ResultWithDefaultError<T> {
        let result =
            V9ApiClient::send::<T>(self.http_client.patch(&url), self.is_official_service).await;
        if result.is_ok() {
            self.invalidate_caches_for_mutation(&url);
        }
        result
    }

    async fn send<T: de::DeserializeOwned>(
        request: RequestBuilder,
        is_official_service: bool,
    ) -> ResultWithDefaultError<T> {
        match request.send().await {
            Err(error) => Err(Box::new(ApiError::NetworkWithMessage(error.to_string()))),
            Ok(response) => {
                let status = response.status();
                let body = response.text().await.map_err(|error| {
                    Box::new(ApiError::NetworkWithMessage(error.to_string()))
                        as Box<dyn std::error::Error + Send>
                })?;

                if !status.is_success() {
                    return Err(Box::new(api_error_for_http_status(
                        status,
                        &body,
                        is_official_service,
                    )));
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
        match self.http_client.delete(&url).send().await {
            Err(error) => Err(Box::new(ApiError::NetworkWithMessage(error.to_string()))),
            Ok(response) => {
                if response.status().is_success() {
                    self.invalidate_caches_for_mutation(&url);
                    Ok(())
                } else {
                    let status = response.status();
                    let body = response.text().await.map_err(|error| {
                        Box::new(ApiError::NetworkWithMessage(error.to_string()))
                            as Box<dyn std::error::Error + Send>
                    })?;
                    Err(Box::new(api_error_for_http_status(
                        status,
                        &body,
                        self.is_official_service,
                    )))
                }
            }
        }
    }
}

fn deserialize_response_body<T: de::DeserializeOwned>(body: &str) -> ResultWithDefaultError<T> {
    serde_json::from_str::<T>(body).map_err(|error| {
        Box::new(ApiError::DeserializationWithMessage(format!(
            "{}; response body: {}",
            error,
            summarize_response_body(body)
        ))) as Box<dyn std::error::Error + Send>
    })
}

fn deserialize_optional_response_body<T: de::DeserializeOwned>(
    body: &str,
) -> ResultWithDefaultError<Option<T>> {
    let trimmed = body.trim();
    if trimmed.is_empty() || trimmed == "null" {
        return Ok(None);
    }

    serde_json::from_str::<T>(trimmed)
        .map(Some)
        .map_err(|error| {
            Box::new(ApiError::DeserializationWithMessage(format!(
                "{}; response body: {}",
                error,
                summarize_response_body(trimmed)
            ))) as Box<dyn std::error::Error + Send>
        })
}

fn cache_root_dir() -> Option<PathBuf> {
    directories::ProjectDirs::from("com.github", "CorrectRoadH", "toggl-cli")
        .map(|dirs| dirs.cache_dir().join("http"))
}

fn stable_hash(value: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

fn cache_namespace_for_token(token: &str) -> String {
    format!("{:x}", stable_hash(token))
}

fn is_http_cache_disabled() -> bool {
    matches!(
        std::env::var("TOGGL_DISABLE_HTTP_CACHE"),
        Ok(value) if matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES")
    )
}

fn cache_ttl_seconds() -> i64 {
    std::env::var("TOGGL_HTTP_CACHE_TTL_SECONDS")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value >= 0)
        .unwrap_or(30)
}

fn cache_ttl_seconds_for_url(url: &str) -> i64 {
    // Check for specific TTL environment variables first
    if url.contains("/me/preferences") || url.contains("/me") && url.ends_with("/me") {
        // User profile data changes rarely
        return std::env::var("TOGGL_HTTP_CACHE_TTL_USER_PROFILE_SECONDS")
            .ok()
            .and_then(|value| value.parse::<i64>().ok())
            .filter(|value| *value >= 0)
            .unwrap_or(300); // 5 minutes default
    }

    if url.contains("/organizations/") && !url.contains("/workspaces") {
        // Organization data changes rarely
        return std::env::var("TOGGL_HTTP_CACHE_TTL_ORGANIZATIONS_SECONDS")
            .ok()
            .and_then(|value| value.parse::<i64>().ok())
            .filter(|value| *value >= 0)
            .unwrap_or(180); // 3 minutes default
    }

    if url.contains("/me/workspaces") || url.contains("/workspaces/") && url.contains("/tags") {
        // Workspace and tags data changes less frequently
        return std::env::var("TOGGL_HTTP_CACHE_TTL_WORKSPACES_SECONDS")
            .ok()
            .and_then(|value| value.parse::<i64>().ok())
            .filter(|value| *value >= 0)
            .unwrap_or(120); // 2 minutes default
    }

    if url.contains("/me/projects") || url.contains("/me/clients") || url.contains("/me/tasks") {
        // Project-related data changes moderately
        return std::env::var("TOGGL_HTTP_CACHE_TTL_PROJECTS_SECONDS")
            .ok()
            .and_then(|value| value.parse::<i64>().ok())
            .filter(|value| *value >= 0)
            .unwrap_or(60); // 1 minute default
    }

    if url.contains("/time_entries") {
        // Time entries change frequently
        return std::env::var("TOGGL_HTTP_CACHE_TTL_TIME_ENTRIES_SECONDS")
            .ok()
            .and_then(|value| value.parse::<i64>().ok())
            .filter(|value| *value >= 0)
            .unwrap_or(15); // 15 seconds default
    }

    // Default TTL for other endpoints
    cache_ttl_seconds()
}

fn is_cacheable_get_url(url: &str) -> bool {
    if url.ends_with("/me")
        || url.ends_with("/me/projects")
        || url.ends_with("/me/tasks")
        || url.ends_with("/me/clients")
        || url.ends_with("/me/workspaces")
        || url.ends_with("/me/organizations")
        || url.ends_with("/me/preferences")
        || url.contains("/me/time_entries")
    {
        return true;
    }

    if url.contains("/organizations/")
        && !url.contains("/organizations//")
        && !url.ends_with("/workspaces")
    {
        return true;
    }

    url.contains("/workspaces/") && url.ends_with("/tags")
}

struct CacheInvalidation {
    exact_urls: Vec<String>,
    invalidate_time_entries: bool,
    bypass_related_endpoints: Vec<String>, // New field for endpoints that should bypass cache temporarily
}

fn cache_invalidation_for_mutation(base_url: &str, mutation_url: &str) -> CacheInvalidation {
    if mutation_url == format!("{base_url}/me/preferences") {
        return CacheInvalidation {
            exact_urls: vec![format!("{base_url}/me/preferences")],
            invalidate_time_entries: false,
            bypass_related_endpoints: vec!["/me/preferences".to_string()],
        };
    }

    if mutation_url.contains("/time_entries") {
        return CacheInvalidation {
            exact_urls: Vec::new(),
            invalidate_time_entries: true,
            bypass_related_endpoints: Vec::new(), // Time entries use the existing mechanism
        };
    }

    if mutation_url.contains("/organizations/") && mutation_url.ends_with("/workspaces") {
        let organization_id = mutation_url
            .trim_end_matches("/workspaces")
            .rsplit('/')
            .next()
            .unwrap_or_default();
        return CacheInvalidation {
            exact_urls: vec![
                format!("{base_url}/me/workspaces"),
                format!("{base_url}/me/organizations"),
                format!("{base_url}/organizations/{organization_id}"),
            ],
            invalidate_time_entries: false,
            bypass_related_endpoints: vec![
                "/me/workspaces".to_string(),
                "/me/organizations".to_string(),
                format!("/organizations/{organization_id}"),
            ],
        };
    }

    if mutation_url.contains("/workspaces/") && mutation_url.contains("/projects/") {
        return CacheInvalidation {
            exact_urls: vec![
                format!("{base_url}/me/projects"),
                format!("{base_url}/me/tasks"),
            ],
            invalidate_time_entries: false,
            bypass_related_endpoints: vec!["/me/projects".to_string(), "/me/tasks".to_string()],
        };
    }

    if mutation_url.ends_with("/projects") {
        return CacheInvalidation {
            exact_urls: vec![
                format!("{base_url}/me/projects"),
                format!("{base_url}/me/tasks"),
            ],
            invalidate_time_entries: false,
            bypass_related_endpoints: vec!["/me/projects".to_string(), "/me/tasks".to_string()],
        };
    }

    if mutation_url.contains("/workspaces/") && mutation_url.contains("/tasks") {
        return CacheInvalidation {
            exact_urls: vec![format!("{base_url}/me/tasks")],
            invalidate_time_entries: false,
            bypass_related_endpoints: vec!["/me/tasks".to_string()],
        };
    }

    if mutation_url.contains("/workspaces/") && mutation_url.contains("/clients") {
        return CacheInvalidation {
            exact_urls: vec![format!("{base_url}/me/clients")],
            invalidate_time_entries: false,
            bypass_related_endpoints: vec!["/me/clients".to_string()],
        };
    }

    if mutation_url.contains("/workspaces/") && mutation_url.contains("/tags") {
        if let Some(workspace_id) = mutation_url
            .split("/workspaces/")
            .nth(1)
            .and_then(|suffix| suffix.split('/').next())
        {
            return CacheInvalidation {
                exact_urls: vec![format!("{base_url}/workspaces/{workspace_id}/tags")],
                invalidate_time_entries: false,
                bypass_related_endpoints: vec![format!("/workspaces/{workspace_id}/tags")],
            };
        }
    }

    if mutation_url.contains("/workspaces/") && !mutation_url.contains("/time_entries") {
        return CacheInvalidation {
            exact_urls: vec![format!("{base_url}/me/workspaces")],
            invalidate_time_entries: false,
            bypass_related_endpoints: vec!["/me/workspaces".to_string()],
        };
    }

    CacheInvalidation {
        exact_urls: Vec::new(),
        invalidate_time_entries: false,
        bypass_related_endpoints: Vec::new(),
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

fn format_http_status_details(status: reqwest::StatusCode, body: &str) -> String {
    format!("HTTP {} {}", status.as_u16(), summarize_response_body(body))
}

fn is_official_toggl_usage_limit_error(
    status: reqwest::StatusCode,
    body: &str,
    is_official_service: bool,
) -> bool {
    if !is_official_service || status != reqwest::StatusCode::PAYMENT_REQUIRED {
        return false;
    }

    let body_lower = body.to_ascii_lowercase();
    body_lower.contains("hourly limit")
        || body_lower.contains("quota will reset")
        || body_lower.contains("upgrade to a paid plan")
        || body_lower.contains("go to subscriptions page")
}

fn api_error_for_http_status(
    status: reqwest::StatusCode,
    body: &str,
    is_official_service: bool,
) -> ApiError {
    let details = format_http_status_details(status, body);
    if (status == reqwest::StatusCode::TOO_MANY_REQUESTS && is_official_service)
        || is_official_toggl_usage_limit_error(status, body, is_official_service)
    {
        ApiError::OfficialApiUsageLimitWithMessage(details)
    } else if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        ApiError::RateLimitedWithMessage(details)
    } else {
        ApiError::NetworkWithMessage(details)
    }
}

#[async_trait]
impl ApiClient for V9ApiClient {
    async fn get_user(&self) -> ResultWithDefaultError<User> {
        let url = format!("{}/me", self.base_url);
        return self.get::<User>(url).await;
    }

    async fn get_projects_list(&self) -> ResultWithDefaultError<Vec<Project>> {
        Ok(self
            .get_projects()
            .await?
            .into_iter()
            .map(Self::network_project_to_project)
            .collect())
    }

    async fn get_tasks_list(&self) -> ResultWithDefaultError<Vec<Task>> {
        let (network_tasks, network_projects) = tokio::join!(self.get_tasks(), self.get_projects());
        let network_tasks = network_tasks?;
        let network_projects = network_projects?;
        let projects = network_projects
            .into_iter()
            .map(|project| (project.id, Self::network_project_to_project(project)))
            .collect::<HashMap<_, _>>();

        Ok(Self::map_tasks(network_tasks, &projects)
            .into_values()
            .collect())
    }

    async fn get_workspaces_list(&self) -> ResultWithDefaultError<Vec<Workspace>> {
        Ok(self
            .get_workspaces()
            .await?
            .into_iter()
            .map(|workspace| Workspace {
                id: workspace.id,
                name: workspace.name,
                admin: workspace.admin,
            })
            .collect())
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

    async fn get_current_time_entry_minimal(&self) -> ResultWithDefaultError<Option<TimeEntry>> {
        let network_entry = match self.get_current_network_time_entry().await? {
            Some(entry) => entry,
            None => return Ok(None),
        };

        Ok(Some(TimeEntry {
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
            created_with: network_entry.created_with,
        }))
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

    async fn get_time_entries_filtered_minimal(
        &self,
        since: Option<String>,
        until: Option<String>,
    ) -> ResultWithDefaultError<Vec<TimeEntry>> {
        let network_entries = self
            .get_time_entries(since.as_deref(), until.as_deref())
            .await?;
        Ok(network_entries
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
                project: None,
                task: None,
                ..Default::default()
            })
            .collect())
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
        let response_body = self
            .send_raw(self.http_client.post(&url).json(&body), &url)
            .await?;
        let network_tag: NetworkTag = Self::parse_single_or_array(&response_body)?;
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
        let response_body = self
            .send_raw(self.http_client.put(&url).json(&body), &url)
            .await?;
        let network_tag: NetworkTag = Self::parse_single_or_array(&response_body)?;
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

    async fn get_organizations(&self) -> ResultWithDefaultError<Vec<Organization>> {
        let url = format!("{}/me/organizations", self.base_url);
        let organizations = self.get::<Vec<NetworkOrganization>>(url).await?;
        Ok(organizations
            .into_iter()
            .map(Self::network_organization_to_organization)
            .collect())
    }

    async fn get_organization(&self, organization_id: i64) -> ResultWithDefaultError<Organization> {
        let url = format!("{}/organizations/{}", self.base_url, organization_id);
        let organization = self.get::<NetworkOrganization>(url).await?;
        Ok(Self::network_organization_to_organization(organization))
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
        match self.http_client.post(&url).json(&preferences).send().await {
            Err(error) => Err(Box::new(ApiError::NetworkWithMessage(error.to_string()))),
            Ok(response) => {
                let status = response.status();
                let body = response.text().await.map_err(|error| {
                    Box::new(ApiError::NetworkWithMessage(error.to_string()))
                        as Box<dyn std::error::Error + Send>
                })?;

                if !status.is_success() {
                    return Err(Box::new(api_error_for_http_status(
                        status,
                        &body,
                        self.is_official_service,
                    )));
                }

                if body.trim().is_empty() {
                    self.invalidate_caches_for_mutation(&url);
                    return Ok(preferences);
                }

                let result = serde_json::from_str::<Value>(&body).map_err(|error| {
                    Box::new(ApiError::DeserializationWithMessage(format!(
                        "{}; response body: {}",
                        error,
                        summarize_response_body(&body)
                    ))) as Box<dyn std::error::Error + Send>
                });

                if result.is_ok() {
                    self.invalidate_caches_for_mutation(&url);
                }
                result
            }
        }
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

    async fn get_summary_report(
        &self,
        workspace_id: i64,
        body: Value,
    ) -> ResultWithDefaultError<Value> {
        let url = format!(
            "{}/workspace/{}/summary/time_entries",
            self.reports_base_url(),
            workspace_id
        );
        let response_body = self
            .send_raw(self.http_client.post(&url).json(&body), &url)
            .await?;
        serde_json::from_str(&response_body).map_err(|e| {
            Box::new(ApiError::NetworkWithMessage(format!(
                "{e}; response body: {response_body}"
            ))) as Box<dyn std::error::Error + Send>
        })
    }

    async fn get_detailed_report(
        &self,
        workspace_id: i64,
        body: Value,
    ) -> ResultWithDefaultError<Value> {
        let url = format!(
            "{}/workspace/{}/search/time_entries",
            self.reports_base_url(),
            workspace_id
        );
        let response_body = self
            .send_raw(self.http_client.post(&url).json(&body), &url)
            .await?;
        serde_json::from_str(&response_body).map_err(|e| {
            Box::new(ApiError::NetworkWithMessage(format!(
                "{e}; response body: {response_body}"
            ))) as Box<dyn std::error::Error + Send>
        })
    }

    async fn get_weekly_report(
        &self,
        workspace_id: i64,
        body: Value,
    ) -> ResultWithDefaultError<Value> {
        let url = format!(
            "{}/workspace/{}/weekly/time_entries",
            self.reports_base_url(),
            workspace_id
        );
        let response_body = self
            .send_raw(self.http_client.post(&url).json(&body), &url)
            .await?;
        serde_json::from_str(&response_body).map_err(|e| {
            Box::new(ApiError::NetworkWithMessage(format!(
                "{e}; response body: {response_body}"
            ))) as Box<dyn std::error::Error + Send>
        })
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

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::StatusCode;

    #[test]
    fn maps_429_to_official_usage_limit_error_for_official_service() {
        let error =
            api_error_for_http_status(StatusCode::TOO_MANY_REQUESTS, "too many requests", true);

        assert_eq!(
            error,
            ApiError::OfficialApiUsageLimitWithMessage("HTTP 429 too many requests".to_string())
        );
    }

    #[test]
    fn keeps_429_as_generic_rate_limit_for_self_hosted_service() {
        let error =
            api_error_for_http_status(StatusCode::TOO_MANY_REQUESTS, "too many requests", false);

        assert_eq!(
            error,
            ApiError::RateLimitedWithMessage("HTTP 429 too many requests".to_string())
        );
    }

    #[test]
    fn maps_hourly_limit_402_to_official_usage_limit_error() {
        let body =
            "You have hit your hourly limit for API calls. Your quota will reset in 1838 seconds.";
        let error = api_error_for_http_status(StatusCode::PAYMENT_REQUIRED, body, true);

        assert_eq!(
            error,
            ApiError::OfficialApiUsageLimitWithMessage(format!("HTTP 402 {body}"))
        );
    }

    #[test]
    fn keeps_non_limit_402_as_network_error() {
        let error =
            api_error_for_http_status(StatusCode::PAYMENT_REQUIRED, "payment required", true);

        assert_eq!(
            error,
            ApiError::NetworkWithMessage("HTTP 402 payment required".to_string())
        );
    }
}
