use crate::api::client::ApiClient;
use crate::error::ArgumentError;
use crate::models::ResultWithDefaultError;
use chrono::{Datelike, Local, NaiveDate};
use colored::Colorize;
use serde_json::{json, Value};

/// Resolve a natural language date string to YYYY-MM-DD format.
/// Mirrors report.rs::resolve_report_date but kept local to avoid cross-module coupling.
fn resolve_search_date(input: &str) -> ResultWithDefaultError<String> {
    let value = input.trim().to_lowercase();
    let today = Local::now().date_naive();

    let date = match value.as_str() {
        "today" | "now" => today,
        "yesterday" => today.pred_opt().unwrap(),
        "this_week" => {
            let days_since_monday = today.weekday().num_days_from_monday();
            today - chrono::Duration::days(days_since_monday as i64)
        }
        "last_week" => {
            let days_since_monday = today.weekday().num_days_from_monday();
            today - chrono::Duration::days((days_since_monday + 7) as i64)
        }
        _ => match NaiveDate::parse_from_str(&value, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => {
                return Err(Box::new(ArgumentError::InvalidReportDate(
                    input.trim().to_string(),
                )));
            }
        },
    };

    Ok(date.format("%Y-%m-%d").to_string())
}

/// Default date window for `entry search`: 1 year back to today.
/// Search is a "find-that-thing-I-did-a-while-ago" tool, so the default needs to be wide.
fn resolve_search_dates(
    since: Option<String>,
    until: Option<String>,
) -> ResultWithDefaultError<(String, String)> {
    let today = Local::now().date_naive();

    let since_resolved = match since {
        Some(s) => resolve_search_date(&s)?,
        None => (today - chrono::Duration::days(365))
            .format("%Y-%m-%d")
            .to_string(),
    };
    let until_resolved = match until {
        Some(u) => resolve_search_date(&u)?,
        None => today.format("%Y-%m-%d").to_string(),
    };

    if since_resolved > until_resolved {
        return Err(Box::new(ArgumentError::InvalidTimeRange(format!(
            "--since ({}) is after --until ({}).",
            since_resolved, until_resolved
        ))));
    }
    Ok((since_resolved, until_resolved))
}

/// Resolve a project name-or-ID string to a numeric ID within the given workspace.
/// Mirrors the resolution pattern used elsewhere in the codebase: try exact name match
/// first, then fall back to numeric ID parsing. Lists available projects on failure.
async fn resolve_project_id(
    api_client: &impl ApiClient,
    workspace_id: i64,
    name_or_id: &str,
) -> ResultWithDefaultError<i64> {
    let projects = api_client.get_projects_list().await?;
    let in_workspace: Vec<_> = projects
        .iter()
        .filter(|p| p.workspace_id == workspace_id)
        .collect();

    if let Some(p) = in_workspace.iter().find(|p| p.name == name_or_id) {
        return Ok(p.id);
    }
    if let Ok(id) = name_or_id.parse::<i64>() {
        if in_workspace.iter().any(|p| p.id == id) {
            return Ok(id);
        }
    }
    let available: Vec<String> = in_workspace
        .iter()
        .map(|p| format!("  - {} (id: {})", p.name, p.id))
        .collect();
    let msg = if available.is_empty() {
        format!("No project found with name or ID '{}'.", name_or_id)
    } else {
        format!(
            "No project found with name or ID '{}'. Available projects:\n{}",
            name_or_id,
            available.join("\n")
        )
    };
    Err(Box::new(ArgumentError::ResourceNotFound(msg)))
}

/// Resolve a tag name to a numeric ID. Tags are workspace-scoped.
async fn resolve_tag_ids(
    api_client: &impl ApiClient,
    workspace_id: i64,
    names: &[String],
) -> ResultWithDefaultError<Vec<i64>> {
    let tags = api_client.get_tags(workspace_id).await?;
    let mut ids = Vec::with_capacity(names.len());
    for name in names {
        match tags.iter().find(|t| t.name == *name) {
            Some(t) => ids.push(t.id),
            None => {
                // Fall back to numeric ID
                if let Ok(id) = name.parse::<i64>() {
                    if tags.iter().any(|t| t.id == id) {
                        ids.push(id);
                        continue;
                    }
                }
                let available: Vec<String> =
                    tags.iter().map(|t| format!("  - {}", t.name)).collect();
                let msg = if available.is_empty() {
                    format!("No tag found with name '{}'.", name)
                } else {
                    format!(
                        "No tag found with name '{}'. Available tags:\n{}",
                        name,
                        available.join("\n")
                    )
                };
                return Err(Box::new(ArgumentError::ResourceNotFound(msg)));
            }
        }
    }
    Ok(ids)
}

#[allow(clippy::too_many_arguments)]
pub async fn execute(
    api_client: impl ApiClient,
    query: Option<String>,
    project: Option<String>,
    no_project: bool,
    tags: Option<Vec<String>>,
    no_tag: bool,
    since: Option<String>,
    until: Option<String>,
    number: Option<i64>,
    order_by: Option<String>,
    order_dir: Option<String>,
    json_output: bool,
) -> ResultWithDefaultError<()> {
    let user = api_client.get_user().await?;
    let workspace_id = user.default_workspace_id;

    let (since, until) = resolve_search_dates(since, until)?;

    let mut body = json!({
        "start_date": since,
        "end_date": until,
    });

    if let Some(q) = query.as_ref().filter(|q| !q.is_empty()) {
        body["description"] = Value::String(q.clone());
    }

    // Project filter: --no-project -> [null]; -p NAME|ID -> [resolved_id]
    if no_project {
        body["project_ids"] = json!([null]);
    } else if let Some(ref p) = project {
        let id = resolve_project_id(&api_client, workspace_id, p).await?;
        body["project_ids"] = json!([id]);
    }

    // Tag filter: --no-tag -> [null]; -t NAME... -> [ids]
    if no_tag {
        body["tag_ids"] = json!([null]);
    } else if let Some(ref names) = tags {
        if !names.is_empty() {
            let ids = resolve_tag_ids(&api_client, workspace_id, names).await?;
            body["tag_ids"] = Value::Array(ids.into_iter().map(|i| json!(i)).collect());
        }
    }

    if let Some(n) = number {
        body["page_size"] = json!(n);
    }
    if let Some(ref ob) = order_by {
        body["order_by"] = Value::String(ob.clone());
    }
    if let Some(ref od) = order_dir {
        body["order_dir"] = Value::String(od.clone());
    }

    let result = api_client.get_detailed_report(workspace_id, body).await?;

    if json_output {
        println!(
            "{}",
            serde_json::to_string(&result).expect("failed to serialize search result to JSON")
        );
    } else {
        print_search_result(&result, &since, &until, query.as_deref());
    }

    Ok(())
}

fn format_duration_hms(total_seconds: i64) -> String {
    let h = total_seconds / 3600;
    let m = (total_seconds % 3600) / 60;
    let s = total_seconds % 60;
    format!("{h}:{m:02}:{s:02}")
}

fn print_search_result(data: &Value, since: &str, until: &str, query: Option<&str>) {
    let header = match query {
        Some(q) if !q.is_empty() => format!("Search results for \"{q}\" ({since} to {until})"),
        _ => format!("Search results ({since} to {until})"),
    };
    println!("{}", header.bold());
    println!("{}", "=".repeat(70));

    let entries: Vec<&Value> = data
        .get("report")
        .and_then(|r| r.as_array())
        .or_else(|| data.get("time_entries").and_then(|v| v.as_array()))
        .or_else(|| data.as_array())
        .map(|arr| arr.iter().collect())
        .unwrap_or_default();

    if entries.is_empty() {
        println!("{}", "No entries found.".dimmed());
        return;
    }

    let mut total_seconds: i64 = 0;
    let mut entry_count: usize = 0;

    for entry in &entries {
        let description = entry
            .get("description")
            .and_then(|d| d.as_str())
            .unwrap_or("(no description)");
        let project_name = entry
            .get("project_name")
            .and_then(|p| p.as_str())
            .unwrap_or("");
        let billable = entry
            .get("billable")
            .and_then(|b| b.as_bool())
            .unwrap_or(false);

        if let Some(nested) = entry.get("time_entries").and_then(|te| te.as_array()) {
            for te in nested {
                let seconds = te.get("seconds").and_then(|s| s.as_i64()).unwrap_or(0);
                let start = te.get("start").and_then(|s| s.as_str()).unwrap_or("");
                let id = te.get("id").and_then(|v| v.as_i64());
                total_seconds += seconds;
                entry_count += 1;
                print_row(id, start, seconds, billable, description, project_name);
            }
        } else {
            let seconds = entry.get("seconds").and_then(|s| s.as_i64()).unwrap_or(0);
            let start = entry.get("start").and_then(|s| s.as_str()).unwrap_or("");
            let id = entry.get("id").and_then(|v| v.as_i64());
            total_seconds += seconds;
            entry_count += 1;
            print_row(id, start, seconds, billable, description, project_name);
        }
    }

    println!("{}", "-".repeat(70));
    println!(
        "  {} {} ({} entries)",
        format_duration_hms(total_seconds).green().bold(),
        "Total".bold(),
        entry_count,
    );
}

fn print_row(
    id: Option<i64>,
    start: &str,
    seconds: i64,
    billable: bool,
    description: &str,
    project_name: &str,
) {
    let duration = format_duration_hms(seconds);
    let billable_marker = if billable { "$" } else { " " };
    let project_display = if project_name.is_empty() {
        String::new()
    } else {
        format!(" @{}", project_name.cyan())
    };
    let date_display = if start.len() >= 10 {
        &start[..10]
    } else {
        start
    };
    let id_display = match id {
        Some(i) => format!("{i:>10} "),
        None => "           ".to_string(),
    };

    println!(
        "{}{} {} [{}] {}{}",
        id_display.dimmed(),
        billable_marker.green().bold(),
        date_display.dimmed(),
        duration.green(),
        description,
        project_display,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_dates_default_is_one_year_window() {
        let (since, until) = resolve_search_dates(None, None).unwrap();
        let today = Local::now().date_naive();
        let one_year_ago = today - chrono::Duration::days(365);
        assert_eq!(since, one_year_ago.format("%Y-%m-%d").to_string());
        assert_eq!(until, today.format("%Y-%m-%d").to_string());
    }

    #[test]
    fn search_dates_respects_explicit_values() {
        let (since, until) = resolve_search_dates(
            Some("2025-01-01".to_string()),
            Some("2025-06-30".to_string()),
        )
        .unwrap();
        assert_eq!(since, "2025-01-01");
        assert_eq!(until, "2025-06-30");
    }

    #[test]
    fn search_dates_rejects_inverted_range() {
        let result = resolve_search_dates(
            Some("2025-06-30".to_string()),
            Some("2025-01-01".to_string()),
        );
        assert!(result.is_err());
    }

    #[test]
    fn search_date_natural_language() {
        let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
        assert_eq!(resolve_search_date("today").unwrap(), today);
    }
}
