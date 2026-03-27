use crate::api::client::ApiClient;
use crate::arguments::ReportAction;
use crate::error::ArgumentError;
use crate::models::ResultWithDefaultError;
use chrono::{Datelike, Local, NaiveDate};
use colored::Colorize;
use serde_json::{json, Value};

/// Resolve a natural language date string to YYYY-MM-DD format.
/// Supports: today, yesterday, now, this_week (Monday of current week),
/// last_week (Monday of last week), or a literal YYYY-MM-DD date.
fn resolve_report_date(input: &str) -> ResultWithDefaultError<String> {
    let value = input.trim().to_lowercase();
    let today = Local::now().date_naive();

    let date = match value.as_str() {
        "today" | "now" => today,
        "yesterday" => today.pred_opt().unwrap(),
        "this_week" => {
            // Monday of the current week
            let days_since_monday = today.weekday().num_days_from_monday();
            today - chrono::Duration::days(days_since_monday as i64)
        }
        "last_week" => {
            let days_since_monday = today.weekday().num_days_from_monday();
            today - chrono::Duration::days((days_since_monday + 7) as i64)
        }
        _ => {
            // Try parsing as YYYY-MM-DD
            match NaiveDate::parse_from_str(&value, "%Y-%m-%d") {
                Ok(d) => d,
                Err(_) => {
                    return Err(Box::new(ArgumentError::InvalidReportDate(
                        input.trim().to_string(),
                    )));
                }
            }
        }
    };

    Ok(date.format("%Y-%m-%d").to_string())
}

/// Resolve --since and --until for report commands, applying defaults
/// (since → Monday of current week, until → today) when not provided.
fn resolve_report_dates(
    since: Option<String>,
    until: Option<String>,
) -> ResultWithDefaultError<(String, String)> {
    let today = Local::now().date_naive();

    let since_resolved = match since {
        Some(s) => resolve_report_date(&s)?,
        None => {
            // Default: Monday of current week
            let days_since_monday = today.weekday().num_days_from_monday();
            let monday = today - chrono::Duration::days(days_since_monday as i64);
            monday.format("%Y-%m-%d").to_string()
        }
    };

    let until_resolved = match until {
        Some(u) => resolve_report_date(&u)?,
        None => today.format("%Y-%m-%d").to_string(),
    };

    // Validate that since <= until
    if since_resolved > until_resolved {
        return Err(Box::new(ArgumentError::InvalidTimeRange(format!(
            "--since ({}) is after --until ({}). Swap the values or use a valid range.",
            since_resolved, until_resolved
        ))));
    }

    Ok((since_resolved, until_resolved))
}

fn format_duration_hms(total_seconds: i64) -> String {
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    format!("{hours}:{minutes:02}:{seconds:02}")
}

pub async fn execute_report_command(
    action: ReportAction,
    api_client: impl ApiClient,
) -> ResultWithDefaultError<()> {
    let user = api_client.get_user().await?;
    let workspace_id = user.default_workspace_id;

    match action {
        ReportAction::Summary {
            since,
            until,
            json,
            group_by,
            sub_group_by,
        } => {
            let (since, until) = resolve_report_dates(since, until)?;
            let mut body = json!({
                "start_date": since,
                "end_date": until,
            });
            if let Some(ref g) = group_by {
                body["grouping"] = Value::String(g.clone());
            }
            if let Some(ref sg) = sub_group_by {
                body["sub_grouping"] = Value::String(sg.clone());
            }

            let result = api_client.get_summary_report(workspace_id, body).await?;

            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&result)
                        .expect("failed to serialize report to JSON")
                );
            } else {
                print_summary_report(&result, &since, &until);
            }
        }
        ReportAction::Detailed {
            since,
            until,
            json,
            number,
            order_by,
            order_dir,
        } => {
            let (since, until) = resolve_report_dates(since, until)?;
            let mut body = json!({
                "start_date": since,
                "end_date": until,
            });
            if let Some(n) = number {
                body["page_size"] = Value::Number(serde_json::Number::from(n));
            }
            if let Some(ref ob) = order_by {
                body["order_by"] = Value::String(ob.clone());
            }
            if let Some(ref od) = order_dir {
                body["order_dir"] = Value::String(od.clone());
            }

            let result = api_client.get_detailed_report(workspace_id, body).await?;

            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&result)
                        .expect("failed to serialize report to JSON")
                );
            } else {
                print_detailed_report(&result, &since, &until);
            }
        }
        ReportAction::Weekly { since, until, json } => {
            let (since, until) = resolve_report_dates(since, until)?;
            let body = json!({
                "start_date": since,
                "end_date": until,
            });

            let result = api_client.get_weekly_report(workspace_id, body).await?;

            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&result)
                        .expect("failed to serialize report to JSON")
                );
            } else {
                print_weekly_report(&result, &since, &until);
            }
        }
    }

    Ok(())
}

/// Extract groups from various response shapes:
/// - `{ "report": { "groups": [...] } }` (OpenToggl)
/// - `{ "groups": [...] }` (official Toggl)
/// - `[...]` (direct array)
fn extract_groups(data: &Value) -> Option<&Vec<Value>> {
    data.get("report")
        .and_then(|r| r.get("groups"))
        .and_then(|g| g.as_array())
        .or_else(|| data.get("groups").and_then(|g| g.as_array()))
        .or_else(|| data.as_array())
}

fn print_summary_report(data: &Value, since: &str, until: &str) {
    println!("{}", format!("Summary Report: {since} to {until}").bold());
    println!("{}", "=".repeat(50));

    let mut grand_total: i64 = 0;

    if let Some(groups) = extract_groups(data) {
        for group in groups {
            let group_name = group
                .get("names")
                .and_then(|n| n.as_array())
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .unwrap_or("(unknown)");

            // sub_groups can be an object (keyed by user/entry ID) or an array
            let mut group_seconds: i64 = 0;
            if let Some(sg_obj) = group.get("sub_groups").and_then(|sg| sg.as_object()) {
                for (_key, sg) in sg_obj {
                    let secs = sg.get("seconds").and_then(|s| s.as_i64()).unwrap_or(0);
                    group_seconds += secs;
                }
            } else if let Some(sg_arr) = group.get("sub_groups").and_then(|sg| sg.as_array()) {
                for sg in sg_arr {
                    let secs = sg.get("seconds").and_then(|s| s.as_i64()).unwrap_or(0);
                    group_seconds += secs;
                }
            }

            grand_total += group_seconds;
            let duration = format_duration_hms(group_seconds);

            let group_id = group.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
            if group_id > 0 {
                println!("  {} {}", duration.green(), group_name);
            } else {
                println!("  {} {}", duration.green(), group_name.dimmed());
            }
        }
    }

    // Also check for totals in the response
    if let Some(totals_seconds) = data
        .get("totals")
        .and_then(|t| t.get("seconds"))
        .and_then(|s| s.as_i64())
    {
        if grand_total == 0 {
            grand_total = totals_seconds;
        }
    }

    println!("{}", "-".repeat(50));
    println!(
        "  {} {}",
        format_duration_hms(grand_total).green().bold(),
        "Total".bold()
    );
}

fn print_detailed_report(data: &Value, since: &str, until: &str) {
    println!("{}", format!("Detailed Report: {since} to {until}").bold());
    println!("{}", "=".repeat(70));

    let mut total_seconds: i64 = 0;
    let mut entry_count: usize = 0;

    // The response might be wrapped in "report" or be a direct array
    let entries: Vec<&Value> = data
        .get("report")
        .and_then(|r| r.as_array())
        .or_else(|| data.get("time_entries").and_then(|v| v.as_array()))
        .or_else(|| data.as_array())
        .map(|arr| arr.iter().collect())
        .unwrap_or_default();

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

        // Time entries can be nested
        if let Some(time_entries) = entry.get("time_entries").and_then(|te| te.as_array()) {
            for te in time_entries {
                let seconds = te.get("seconds").and_then(|s| s.as_i64()).unwrap_or(0);
                let start = te.get("start").and_then(|s| s.as_str()).unwrap_or("");

                total_seconds += seconds;
                entry_count += 1;

                print_detailed_entry(start, seconds, billable, description, project_name);
            }
        } else {
            // Flat structure: seconds directly on the entry
            let seconds = entry.get("seconds").and_then(|s| s.as_i64()).unwrap_or(0);
            let start = entry.get("start").and_then(|s| s.as_str()).unwrap_or("");
            total_seconds += seconds;
            entry_count += 1;

            print_detailed_entry(start, seconds, billable, description, project_name);
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

fn print_detailed_entry(
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

    println!(
        "{} {} [{}] {}{}",
        billable_marker.green().bold(),
        date_display.dimmed(),
        duration.green(),
        description,
        project_display,
    );
}

fn print_weekly_report(data: &Value, since: &str, until: &str) {
    println!("{}", format!("Weekly Report: {since} to {until}").bold());
    println!("{}", "=".repeat(100));

    println!(
        "  {:<30} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8}",
        "Project / User".bold(),
        "Mon",
        "Tue",
        "Wed",
        "Thu",
        "Fri",
        "Sat",
        "Sun",
        "Total"
    );
    println!("{}", "-".repeat(100));

    let rows: Vec<&Value> = data
        .get("report")
        .and_then(|r| r.as_array())
        .or_else(|| data.get("rows").and_then(|v| v.as_array()))
        .or_else(|| data.as_array())
        .map(|arr| arr.iter().collect())
        .unwrap_or_default();

    for row in &rows {
        let project_name = row
            .get("project_name")
            .and_then(|p| p.as_str())
            .or_else(|| row.get("user_name").and_then(|u| u.as_str()))
            .unwrap_or("(unknown)");

        let seconds_arr = row
            .get("seconds")
            .and_then(|s| s.as_array())
            .cloned()
            .unwrap_or_default();

        let mut row_total: i64 = 0;
        let mut day_strings: Vec<String> = Vec::new();

        for day_val in &seconds_arr {
            let secs = day_val.as_i64().unwrap_or(0);
            row_total += secs;
            if secs > 0 {
                day_strings.push(format_duration_hms(secs));
            } else {
                day_strings.push("-".to_string());
            }
        }

        while day_strings.len() < 7 {
            day_strings.push("-".to_string());
        }

        let total_str = if row_total > 0 {
            format_duration_hms(row_total)
        } else {
            "-".to_string()
        };

        let display_name: String = if project_name.len() > 28 {
            format!("{}...", &project_name[..25])
        } else {
            project_name.to_string()
        };

        println!(
            "  {:<30} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8}",
            display_name,
            day_strings[0],
            day_strings[1],
            day_strings[2],
            day_strings[3],
            day_strings[4],
            day_strings[5],
            day_strings[6],
            total_str.green().bold(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::client::MockApiClient;
    use crate::error::ApiError;
    use crate::models::User;
    use serde_json::json;
    use tokio_test::{assert_err, assert_ok};

    fn mock_user() -> User {
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

    // --- resolve_report_date tests ---

    #[test]
    fn resolve_report_date_today() {
        let result = resolve_report_date("today").unwrap();
        let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
        assert_eq!(result, today);
    }

    #[test]
    fn resolve_report_date_yesterday() {
        let result = resolve_report_date("yesterday").unwrap();
        let yesterday = Local::now()
            .date_naive()
            .pred_opt()
            .unwrap()
            .format("%Y-%m-%d")
            .to_string();
        assert_eq!(result, yesterday);
    }

    #[test]
    fn resolve_report_date_now() {
        let result = resolve_report_date("now").unwrap();
        let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
        assert_eq!(result, today);
    }

    #[test]
    fn resolve_report_date_this_week() {
        let result = resolve_report_date("this_week").unwrap();
        let today = Local::now().date_naive();
        let days_since_monday = today.weekday().num_days_from_monday();
        let monday = today - chrono::Duration::days(days_since_monday as i64);
        assert_eq!(result, monday.format("%Y-%m-%d").to_string());
    }

    #[test]
    fn resolve_report_date_last_week() {
        let result = resolve_report_date("last_week").unwrap();
        let today = Local::now().date_naive();
        let days_since_monday = today.weekday().num_days_from_monday();
        let last_monday = today - chrono::Duration::days((days_since_monday + 7) as i64);
        assert_eq!(result, last_monday.format("%Y-%m-%d").to_string());
    }

    #[test]
    fn resolve_report_date_valid_yyyy_mm_dd() {
        let result = resolve_report_date("2026-03-15").unwrap();
        assert_eq!(result, "2026-03-15");
    }

    #[test]
    fn resolve_report_date_case_insensitive() {
        let result = resolve_report_date("TODAY").unwrap();
        let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
        assert_eq!(result, today);
    }

    #[test]
    fn resolve_report_date_invalid() {
        let result = resolve_report_date("baddate");
        assert!(result.is_err());
    }

    // --- resolve_report_dates tests ---

    #[test]
    fn resolve_report_dates_defaults_to_this_week() {
        let (since, until) = resolve_report_dates(None, None).unwrap();
        let today = Local::now().date_naive();
        let days_since_monday = today.weekday().num_days_from_monday();
        let monday = today - chrono::Duration::days(days_since_monday as i64);
        assert_eq!(since, monday.format("%Y-%m-%d").to_string());
        assert_eq!(until, today.format("%Y-%m-%d").to_string());
    }

    #[test]
    fn resolve_report_dates_since_after_until_errors() {
        let result = resolve_report_dates(
            Some("2026-03-20".to_string()),
            Some("2026-03-10".to_string()),
        );
        assert!(result.is_err());
    }

    #[test]
    fn resolve_report_dates_same_day_ok() {
        let (since, until) = resolve_report_dates(
            Some("2026-03-15".to_string()),
            Some("2026-03-15".to_string()),
        )
        .unwrap();
        assert_eq!(since, "2026-03-15");
        assert_eq!(until, "2026-03-15");
    }

    // --- format_duration_hms tests ---

    #[test]
    fn format_duration_zero() {
        assert_eq!(format_duration_hms(0), "0:00:00");
    }

    #[test]
    fn format_duration_one_hour() {
        assert_eq!(format_duration_hms(3600), "1:00:00");
    }

    #[test]
    fn format_duration_mixed() {
        assert_eq!(format_duration_hms(3723), "1:02:03");
    }

    // --- execute_report_command integration tests ---

    #[tokio::test]
    async fn summary_report_with_defaults() {
        let mut api_client = MockApiClient::new();
        let user = mock_user();
        api_client
            .expect_get_user()
            .returning(move || Ok(user.clone()));
        api_client
            .expect_get_summary_report()
            .withf(|wid, _body| *wid == 1)
            .returning(|_, _| Ok(json!({"groups": [], "totals": {"seconds": 0}})));

        let result = execute_report_command(
            ReportAction::Summary {
                since: None,
                until: None,
                json: false,
                group_by: None,
                sub_group_by: None,
            },
            api_client,
        )
        .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn summary_report_json_output() {
        let mut api_client = MockApiClient::new();
        let user = mock_user();
        api_client
            .expect_get_user()
            .returning(move || Ok(user.clone()));
        api_client
            .expect_get_summary_report()
            .returning(|_, _| Ok(json!({"groups": [], "totals": {"seconds": 3600}})));

        let result = execute_report_command(
            ReportAction::Summary {
                since: Some("today".to_string()),
                until: Some("today".to_string()),
                json: true,
                group_by: None,
                sub_group_by: None,
            },
            api_client,
        )
        .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn weekly_report_with_natural_dates() {
        let mut api_client = MockApiClient::new();
        let user = mock_user();
        api_client
            .expect_get_user()
            .returning(move || Ok(user.clone()));
        api_client
            .expect_get_weekly_report()
            .withf(|wid, _body| *wid == 1)
            .returning(|_, _| Ok(json!({"rows": []})));

        let result = execute_report_command(
            ReportAction::Weekly {
                since: Some("this_week".to_string()),
                until: Some("today".to_string()),
                json: false,
            },
            api_client,
        )
        .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn detailed_report_with_options() {
        let mut api_client = MockApiClient::new();
        let user = mock_user();
        api_client
            .expect_get_user()
            .returning(move || Ok(user.clone()));
        api_client
            .expect_get_detailed_report()
            .withf(|wid, body| {
                *wid == 1
                    && body.get("page_size").and_then(|v| v.as_i64()) == Some(10)
                    && body.get("order_by").and_then(|v| v.as_str()) == Some("date")
            })
            .returning(|_, _| Ok(json!({"time_entries": []})));

        let result = execute_report_command(
            ReportAction::Detailed {
                since: Some("2026-03-01".to_string()),
                until: Some("2026-03-27".to_string()),
                json: false,
                number: Some(10),
                order_by: Some("date".to_string()),
                order_dir: Some("desc".to_string()),
            },
            api_client,
        )
        .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn report_with_invalid_date_returns_error() {
        let mut api_client = MockApiClient::new();
        let user = mock_user();
        api_client
            .expect_get_user()
            .returning(move || Ok(user.clone()));

        let result = execute_report_command(
            ReportAction::Summary {
                since: Some("baddate".to_string()),
                until: Some("today".to_string()),
                json: false,
                group_by: None,
                sub_group_by: None,
            },
            api_client,
        )
        .await;
        assert_err!(result);
    }

    #[tokio::test]
    async fn report_api_failure_propagates() {
        let mut api_client = MockApiClient::new();
        let user = mock_user();
        api_client
            .expect_get_user()
            .returning(move || Ok(user.clone()));
        api_client
            .expect_get_summary_report()
            .returning(|_, _| Err(Box::new(ApiError::Network)));

        let result = execute_report_command(
            ReportAction::Summary {
                since: Some("today".to_string()),
                until: Some("today".to_string()),
                json: false,
                group_by: None,
                sub_group_by: None,
            },
            api_client,
        )
        .await;
        assert_err!(result);
    }
}
