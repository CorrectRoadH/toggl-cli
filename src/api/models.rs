use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};

use crate::models::TimeEntry;

#[derive(Serialize, Clone, Debug)]
pub struct NetworkTimeEntry {
    pub id: i64,
    pub description: String,
    pub start: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<DateTime<Utc>>,
    pub duration: i64,
    pub billable: bool,
    #[serde(alias = "wid")]
    pub workspace_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(alias = "pid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<i64>,
    #[serde(alias = "tid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_with: Option<String>,
}

#[derive(Deserialize)]
struct RawNetworkTimeEntry {
    id: i64,
    description: String,
    start: DateTime<Utc>,
    stop: Option<DateTime<Utc>>,
    duration: i64,
    billable: bool,
    workspace_id: Option<i64>,
    wid: Option<i64>,
    tags: Option<Vec<String>>,
    project_id: Option<i64>,
    pid: Option<i64>,
    task_id: Option<i64>,
    tid: Option<i64>,
    created_with: Option<String>,
}

impl<'de> Deserialize<'de> for NetworkTimeEntry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawNetworkTimeEntry::deserialize(deserializer)?;
        Ok(Self {
            id: raw.id,
            description: raw.description,
            start: raw.start,
            stop: raw.stop,
            duration: raw.duration,
            billable: raw.billable,
            workspace_id: raw
                .workspace_id
                .or(raw.wid)
                .ok_or_else(|| serde::de::Error::missing_field("workspace_id"))?,
            tags: raw.tags,
            project_id: raw.project_id.or(raw.pid),
            task_id: raw.task_id.or(raw.tid),
            created_with: raw.created_with,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NetworkProject {
    pub id: i64,
    pub name: String,
    pub workspace_id: i64,
    pub client_id: Option<i64>,
    #[serde(default)]
    pub is_private: bool,
    pub active: bool,
    pub at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub server_deleted_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub color: String,
    pub billable: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NetworkClient {
    pub id: i64,
    pub name: String,
    pub wid: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NetworkTask {
    pub id: i64,
    pub name: String,
    pub workspace_id: i64,
    pub project_id: i64,
    #[serde(default)]
    pub active: Option<bool>,
    #[serde(default)]
    pub estimated_seconds: Option<i64>,
    #[serde(default)]
    pub external_reference: Option<String>,
    #[serde(default)]
    pub user_id: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NetworkWorkspace {
    pub id: i64,
    pub name: String,
    pub admin: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NetworkOrganization {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub admin: bool,
    #[serde(default)]
    pub workspace_id: Option<i64>,
    #[serde(default)]
    pub workspace_name: Option<String>,
    #[serde(default)]
    pub pricing_plan_name: Option<String>,
    #[serde(default)]
    pub permissions: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NetworkTag {
    pub id: i64,
    pub name: String,
    pub workspace_id: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NetworkCreateProject {
    pub name: String,
    pub workspace_id: i64,
    pub color: String,
    pub is_private: bool,
    pub active: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NetworkCreateTag {
    pub name: String,
    pub workspace_id: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NetworkRenameTag {
    pub name: String,
    pub workspace_id: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NetworkRenameProject {
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NetworkCreateClient {
    pub name: String,
    pub wid: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NetworkRenameClient {
    pub name: String,
    pub wid: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NetworkCreateWorkspace {
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct NetworkUpdateWorkspace {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NetworkCreateTask {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_seconds: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct NetworkUpdateTask {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_seconds: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<i64>,
}

impl From<TimeEntry> for NetworkTimeEntry {
    fn from(value: TimeEntry) -> Self {
        NetworkTimeEntry {
            id: value.id,
            description: value.description.to_string(),
            start: value.start,
            stop: value.stop,
            duration: value.duration,
            billable: value.billable,
            workspace_id: value.workspace_id,
            tags: if value.tags.is_empty() {
                None
            } else {
                Some(value.tags.clone())
            },
            project_id: value.project.map(|p| p.id),
            task_id: value.task.map(|t| t.id),
            created_with: value.created_with,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{NetworkProject, NetworkTimeEntry};
    use chrono::{DateTime, Utc};

    #[test]
    fn network_time_entry_accepts_both_long_and_short_id_fields() {
        let entry: NetworkTimeEntry = serde_json::from_str(
            r#"{
                "id": 1,
                "description": "entry",
                "start": "2026-03-06T15:09:03Z",
                "stop": null,
                "duration": -1,
                "billable": false,
                "workspace_id": 3550374,
                "wid": 3550374,
                "project_id": 212780915,
                "pid": 212780915,
                "task_id": null,
                "tid": null,
                "tags": ["tag-1"]
            }"#,
        )
        .expect("expected network time entry to deserialize");

        assert_eq!(entry.workspace_id, 3550374);
        assert_eq!(entry.project_id, Some(212780915));
        assert_eq!(entry.task_id, None);
    }

    #[test]
    fn network_time_entry_omits_optional_none_fields_when_serialized() {
        let entry = NetworkTimeEntry {
            id: 1,
            description: "entry".to_string(),
            start: DateTime::parse_from_rfc3339("2026-03-06T15:09:03Z")
                .expect("valid RFC3339")
                .with_timezone(&Utc),
            stop: None,
            duration: -1,
            billable: false,
            workspace_id: 3550374,
            tags: None,
            project_id: None,
            task_id: None,
            created_with: None,
        };

        let serialized =
            serde_json::to_value(entry).expect("expected network time entry to serialize");
        let object = serialized
            .as_object()
            .expect("expected serialized network time entry to be an object");

        assert!(!object.contains_key("stop"));
        assert!(!object.contains_key("tags"));
        assert!(!object.contains_key("project_id"));
        assert!(!object.contains_key("task_id"));
        assert!(!object.contains_key("created_with"));
    }

    #[test]
    fn network_project_defaults_is_private_when_missing() {
        let project: NetworkProject = serde_json::from_str(
            r##"{
                "id": 3,
                "name": "Project",
                "workspace_id": 1,
                "client_id": null,
                "active": true,
                "at": "2026-03-06T15:09:03Z",
                "created_at": "2026-03-06T15:09:03Z",
                "server_deleted_at": null,
                "color": "#000000",
                "billable": null
            }"##,
        )
        .expect("expected network project to deserialize");

        assert!(!project.is_private);
    }
}
