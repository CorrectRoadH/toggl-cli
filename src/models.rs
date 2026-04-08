use std::{cmp, env};

use crate::constants;
use std::collections::HashMap;

use chrono::{DateTime, Duration, Local, Utc};
use colored::{ColoredString, Colorize};
use colors_transform::{Color, Rgb};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

pub type ResultWithDefaultError<T> = Result<T, Box<dyn std::error::Error + Send>>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub workspace_id: i64,
}

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Entities {
    pub time_entries: Vec<TimeEntry>,
    pub projects: HashMap<i64, Project>,
    pub tasks: HashMap<i64, Task>,
    pub clients: HashMap<i64, Client>,
    pub workspaces: Vec<Workspace>,
    pub tags: Vec<Tag>,
}

impl Entities {
    pub fn workspace_id_for_name(&self, name: &str) -> Option<i64> {
        self.workspaces
            .iter()
            .find(|w| w.name == name)
            .map(|w| w.id)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    pub api_token: String,
    pub email: String,
    pub fullname: Option<String>,
    pub timezone: String,
    pub default_workspace_id: i64,
    #[serde(default)]
    pub beginning_of_week: Option<i32>,
    #[serde(default)]
    pub image_url: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub country_id: Option<i64>,
    #[serde(default)]
    pub has_password: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TimeEntry {
    pub id: i64,
    pub description: String,
    pub start: DateTime<Utc>,
    pub stop: Option<DateTime<Utc>>,
    pub duration: i64,
    pub billable: bool,
    #[serde(skip_serializing)]
    pub workspace_id: i64,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<Project>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<Task>,
    #[serde(skip_serializing)]
    pub created_with: Option<String>,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub workspace_id: i64,
    #[serde(skip_serializing)]
    pub client: Option<Client>,
    #[serde(skip_serializing)]
    pub is_private: bool,
    #[serde(skip_serializing)]
    pub active: bool,
    #[serde(skip_serializing)]
    pub at: DateTime<Utc>,
    #[serde(skip_serializing)]
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing)]
    pub color: String,
    #[serde(skip_serializing)]
    pub billable: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Workspace {
    pub id: i64,
    pub name: String,
    pub admin: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Organization {
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
    pub permissions: Vec<String>,
}

lazy_static! {
    pub static ref HAS_TRUECOLOR_SUPPORT: bool = if let Ok(truecolor) = env::var("COLORTERM") {
        truecolor == "truecolor" || truecolor == "24bit"
    } else {
        false
    };
}

impl Project {
    /// Gets the closest plain color to the TrueColor
    pub fn name_in_closest_terminal_color(&self, red: u8, green: u8, blue: u8) -> ColoredString {
        let colors = vec![
            (0, 0, 0),       //Black
            (205, 0, 0),     //Red
            (0, 205, 0),     //Green
            (205, 205, 0),   //Yellow
            (0, 0, 238),     //Blue
            (205, 0, 205),   //Magenta
            (0, 205, 205),   //Cyan
            (229, 229, 229), //White
            (127, 127, 127), //BrightBlack
            (255, 0, 0),     //BrightRed
            (0, 255, 0),     //BrightGreen
            (255, 255, 0),   //BrightYellow
            (92, 92, 255),   //BrightBlue
            (255, 0, 255),   //BrightMagenta
            (0, 255, 255),   //BrightCyan
            (255, 255, 255), //BrightWhite
        ]
        .into_iter()
        .enumerate();

        let index = colors
            .map(|(index, (r, g, b))| {
                let rd = cmp::max(r, red) - cmp::min(r, red);
                let gd = cmp::max(g, green) - cmp::min(g, green);
                let bd = cmp::max(b, blue) - cmp::min(b, blue);
                let rd: u32 = rd.into();
                let gd: u32 = gd.into();
                let bd: u32 = bd.into();
                let distance: u32 = rd.pow(2) + gd.pow(2) + bd.pow(2);
                (distance, index)
            })
            .min_by(|(d1, _), (d2, _)| d1.cmp(d2))
            .unwrap()
            .1;

        match index {
            0 => self.name.black(),
            1 => self.name.red(),
            2 => self.name.green(),
            3 => self.name.yellow(),
            4 => self.name.blue(),
            5 => self.name.magenta(),
            6 => self.name.cyan(),
            7 => self.name.white(),
            8 => self.name.bright_black(),
            9 => self.name.bright_red(),
            10 => self.name.bright_green(),
            11 => self.name.bright_yellow(),
            12 => self.name.bright_blue(),
            13 => self.name.bright_magenta(),
            14 => self.name.bright_cyan(),
            15 => self.name.bright_white(),
            _ => self.name.white().clear(),
        }
    }
}

impl std::fmt::Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let colored_name = match Rgb::from_hex_str(self.color.as_str()) {
            Ok(color) => {
                let red = color.get_red().round() as u8;
                let green = color.get_green().round() as u8;
                let blue = color.get_blue().round() as u8;

                if HAS_TRUECOLOR_SUPPORT.to_owned() {
                    self.name.truecolor(red, green, blue).bold()
                } else {
                    self.name_in_closest_terminal_color(red, green, blue).bold()
                }
            }
            Err(_) => self.name.bold(),
        };
        if self.active {
            write!(f, "{colored_name}")
        } else {
            write!(f, "{colored_name} (archived)")
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Client {
    pub id: i64,
    pub name: String,
    pub workspace_id: i64,
    #[serde(default)]
    pub archived: bool,
}

impl std::fmt::Display for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.archived {
            write!(f, "{} (archived)", self.name)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

impl std::fmt::Display for Workspace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let role = if self.admin { "admin" } else { "member" };
        write!(f, "{} ({})", self.name, role)
    }
}

impl std::fmt::Display for Organization {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let role = if self.admin { "admin" } else { "member" };
        match self.pricing_plan_name.as_deref() {
            Some(plan) if !plan.is_empty() => {
                write!(f, "{} (#{}; {}; plan: {})", self.name, self.id, role, plan)
            }
            _ => write!(f, "{} (#{}; {})", self.name, self.id, role),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Task {
    pub id: i64,
    pub name: String,
    pub workspace_id: i64,
    pub project: Project,
    #[serde(default = "default_true")]
    pub active: bool,
}

fn default_true() -> bool {
    true
}

impl std::fmt::Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.active {
            write!(f, "{} @{}", self.name, self.project.name)
        } else {
            write!(f, "{} @{} (done)", self.name, self.project.name)
        }
    }
}

impl TimeEntry {
    pub fn get_description(&self) -> String {
        match self.description.as_ref() {
            "" => constants::NO_DESCRIPTION.to_string(),
            _ => self.description.to_string(),
        }
    }

    pub fn get_duration(&self) -> Duration {
        match self.stop {
            Some(_) => Duration::seconds(self.duration),
            None => Utc::now().signed_duration_since(self.start),
        }
    }

    pub fn get_duration_hmmss(&self) -> String {
        let duration = self.get_duration();
        format!(
            "{}:{:02}:{:02}",
            duration.num_hours(),
            duration.num_minutes() % 60,
            duration.num_seconds() % 60
        )
    }

    pub fn is_running(&self) -> bool {
        self.duration.is_negative()
    }

    pub fn as_running_time_entry(&self, start: DateTime<Utc>) -> TimeEntry {
        TimeEntry {
            start,
            stop: None,
            duration: -start.timestamp(),
            created_with: Some(constants::CLIENT_NAME.to_string()),
            ..self.clone()
        }
    }

    pub fn get_display_tags(&self) -> String {
        if self.tags.is_empty() {
            "".to_string()
        } else {
            format!("[{}]", self.tags.join(", "))
        }
    }
}

impl Default for TimeEntry {
    fn default() -> Self {
        let start = Utc::now();
        Self {
            id: -1,
            created_with: Some(constants::CLIENT_NAME.to_string()),
            billable: false,
            description: "".to_string(),
            duration: -start.timestamp(),
            project: None,
            start,
            stop: None,
            tags: Vec::new(),
            task: None,
            workspace_id: -1,
        }
    }
}

impl std::fmt::Display for TimeEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let local_start: DateTime<Local> = self.start.with_timezone(&Local);
        let time_str = match self.stop {
            Some(stop) => {
                let local_stop: DateTime<Local> = stop.with_timezone(&Local);
                let cross_day = local_stop.date_naive() != local_start.date_naive();
                if cross_day {
                    format!(
                        "{} ~ {}",
                        local_start.format("%Y-%m-%d %H:%M"),
                        local_stop.format("%m-%d %H:%M")
                    )
                } else {
                    format!(
                        "{}~{}",
                        local_start.format("%Y-%m-%d %H:%M"),
                        local_stop.format("%H:%M")
                    )
                }
            }
            None => format!("{}~…", local_start.format("%Y-%m-%d %H:%M")),
        };
        let summary = format!(
            //{id} {date HH:MM–HH:MM} {$/space} [{duration}]{running indicator/space} – {description}{@project} {#tags}
            "{} {} {} [{}]{} –  {}{} {}",
            self.id,
            time_str,
            if self.billable {
                "$".green().bold().to_string()
            } else {
                " ".to_string()
            },
            if self.is_running() {
                self.get_duration_hmmss().green().bold()
            } else {
                self.get_duration_hmmss().normal()
            },
            if self.is_running() { "*" } else { " " },
            self.get_description().replace('\n', " "),
            match self.project.clone() {
                Some(p) => format!(" @{p}"),
                None => "".to_string(),
            },
            if self.tags.is_empty() {
                "".to_string()
            } else {
                format!("#{}", self.get_display_tags().italic())
            }
        );
        write!(f, "{summary}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn entry_with_start_stop(start_secs: i64, stop_secs: Option<i64>) -> TimeEntry {
        let start = Utc.timestamp_opt(start_secs, 0).single().unwrap();
        TimeEntry {
            id: 1,
            description: "test".to_string(),
            start,
            stop: stop_secs.map(|s| Utc.timestamp_opt(s, 0).single().unwrap()),
            duration: stop_secs.map_or(-start.timestamp(), |s| s - start_secs),
            billable: false,
            workspace_id: 1,
            tags: vec![],
            project: None,
            task: None,
            created_with: None,
        }
    }

    #[test]
    fn display_same_day_entry_shows_tilde_end_time() {
        // 2023-11-14 15:00 UTC ~ 2023-11-14 16:00 UTC
        let entry = entry_with_start_stop(1_700_000_000, Some(1_700_003_600));
        let display = format!("{entry}");
        // Should contain ~ connecting start and end, no date in end part
        assert!(display.contains('~'), "should contain ~ separator");
        // Should NOT contain " ~ " (space-padded) since same day
        assert!(
            !display.contains(" ~ "),
            "same-day should use compact ~ without spaces, got: {display}"
        );
    }

    #[test]
    fn display_cross_day_entry_shows_end_date() {
        // Use local time to construct a pair that's definitely cross-day locally:
        // local 23:00 today ~ local 11:00 tomorrow
        let today = Local::now().date_naive();
        let tomorrow = today.succ_opt().unwrap();
        let start_naive = today.and_hms_opt(23, 0, 0).unwrap();
        let stop_naive = tomorrow.and_hms_opt(11, 0, 0).unwrap();
        let start = Local
            .from_local_datetime(&start_naive)
            .single()
            .unwrap()
            .with_timezone(&Utc);
        let stop = Local
            .from_local_datetime(&stop_naive)
            .single()
            .unwrap()
            .with_timezone(&Utc);
        let entry = TimeEntry {
            id: 1,
            description: "test".to_string(),
            start,
            stop: Some(stop),
            duration: (stop - start).num_seconds(),
            billable: false,
            workspace_id: 1,
            tags: vec![],
            project: None,
            task: None,
            created_with: None,
        };
        let display = format!("{entry}");
        assert!(
            display.contains(" ~ "),
            "cross-day should use spaced ~ separator, got: {display}"
        );
    }

    #[test]
    fn display_running_entry_shows_ellipsis() {
        let today = Local::now().date_naive();
        let start_naive = today.and_hms_opt(9, 0, 0).unwrap();
        let start = Local
            .from_local_datetime(&start_naive)
            .single()
            .unwrap()
            .with_timezone(&Utc);
        let entry = TimeEntry {
            id: 1,
            description: "test".to_string(),
            start,
            stop: None,
            duration: -start.timestamp(),
            billable: false,
            workspace_id: 1,
            tags: vec![],
            project: None,
            task: None,
            created_with: None,
        };
        let display = format!("{entry}");
        assert!(
            display.contains("~…"),
            "running entry should show ~…, got: {display}"
        );
    }
}
