use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// Toggl command line app.
#[derive(Parser, Debug)]
#[command(name = "toggl")]
#[command(about = "Toggl command line app.", long_about = None)]
#[command(subcommand_required = true, arg_required_else_help = true)]
#[command(after_long_help = "\
Examples:
  toggl entry start -d \"Working on feature\"
  toggl entry stop
  toggl entry running
  toggl entry list")]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Command,

    #[arg(short = 'C', help = "Change directory before running the command")]
    pub directory: Option<PathBuf>,

    #[arg(long, help = "Use custom proxy")]
    pub proxy: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Authenticate with Toggl API or check auth status.
    Auth {
        #[command(subcommand)]
        action: Option<AuthAction>,
        #[arg(help = "API token for authentication")]
        api_token: Option<String>,
        #[arg(long, help = "Toggl service type: 'official' or 'opentoggl'")]
        api_type: Option<String>,
        #[arg(long, help = "API URL for self-hosted Toggl (required for opentoggl)")]
        api_url: Option<String>,
    },
    /// Clear stored credentials.
    Logout,
    /// Show current user profile information.
    #[command(after_long_help = "\
Examples:
  toggl me                        Show your profile info
  toggl me --json                 Show your profile as JSON")]
    Me {
        #[arg(short, long, help = "Output in JSON format")]
        json: bool,
    },
    /// Manage time entries.
    #[command(after_long_help = "\
Examples:
  toggl entry start -d \"Working on feature\"
  toggl entry running
  toggl entry stop
  toggl entry list --since yesterday")]
    Entry {
        #[command(subcommand)]
        action: EntryAction,
    },
    /// Manage projects.
    Project {
        #[command(subcommand)]
        action: ProjectAction,
    },
    /// Manage tags.
    Tag {
        #[command(subcommand)]
        action: TagAction,
    },
    /// Manage clients.
    Client {
        #[command(subcommand)]
        action: ClientAction,
    },
    /// Manage tasks.
    Task {
        #[command(subcommand)]
        action: TaskAction,
    },
    /// Manage workspaces.
    Workspace {
        #[command(subcommand)]
        action: WorkspaceAction,
    },
    /// Inspect organizations.
    Org {
        #[command(subcommand)]
        action: OrganizationAction,
    },
    /// Show current user preferences.
    Preferences {
        #[command(subcommand)]
        action: PreferencesAction,
    },
    /// Generate reports (summary, detailed, weekly)
    #[command(after_long_help = "\
Examples:
  toggl report summary
  toggl report summary --since today --until today
  toggl report detailed --since yesterday --until today
  toggl report weekly --since 2026-03-17 --until 2026-03-23
  toggl report detailed --json -n 100")]
    Report {
        #[command(subcommand)]
        action: ReportAction,
    },
    /// Manage configuration.
    Config {
        #[arg(
            short,
            long,
            help = "Edit the configuration file in $EDITOR, defaults to vim"
        )]
        edit: bool,
        #[arg(short, long, help = "Delete the configuration file")]
        delete: bool,
        #[arg(short, long, help = "Print the path of the configuration file")]
        path: bool,
        #[command(subcommand)]
        cmd: Option<ConfigAction>,
    },
}

#[derive(Subcommand, Debug)]
pub enum EntryAction {
    /// Show the currently running time entry.
    #[command(
        name = "running",
        alias = "current",
        after_long_help = "\
Examples:
  toggl entry running             Show the running entry
  toggl entry running --json      Show the running entry as JSON

Note: JSON output includes a \"running\": true field for active entries."
    )]
    Current {
        #[arg(short, long, help = "Output in JSON format")]
        json: bool,
    },
    /// List time entries.
    #[command(after_long_help = "\
Examples:
  toggl entry list
  toggl entry list --since today
  toggl entry list --since yesterday --until today
  toggl entry list --since 2024-01-01 --number 5
  toggl entry list --json | jq '.[].description'")]
    List {
        #[arg(short, long, help = "Maximum number of items to print")]
        number: Option<usize>,
        #[arg(short, long, help = "Output in JSON format")]
        json: bool,
        #[arg(
            long,
            help = "Filter time entries starting on or after this date/time (now, today, yesterday, this_week, last_week, YYYY-MM-DD, or full datetime)"
        )]
        since: Option<String>,
        #[arg(
            long,
            help = "Filter time entries before this date/time (now, today, yesterday, this_week, last_week, YYYY-MM-DD, or full datetime)"
        )]
        until: Option<String>,
    },
    /// Stop the currently running time entry.
    #[command(after_long_help = "\
Examples:
  toggl entry stop                Stop the running entry
  toggl entry stop --json         Stop and output as JSON")]
    Stop {
        #[arg(short, long, help = "Output in JSON format")]
        json: bool,
    },
    /// Start a new time entry (runs immediately with no prompt when called without arguments).
    #[command(after_long_help = "\
Examples:
  toggl entry start -d \"Writing docs\" -p MyProject
  toggl entry start -d \"Meeting\" --start 09:00 --end 10:00
  toggl entry start --json")]
    Start {
        #[arg(short, long, help = "Description of the time entry")]
        description: Option<String>,
        #[arg(
            short,
            long,
            help = "Exact name of the project you want the time entry to be associated with"
        )]
        project: Option<String>,
        #[arg(
            long,
            help = "Exact name of the task you want the time entry to be associated with"
        )]
        task: Option<String>,
        #[arg(short, long, help = "Tag name (repeatable), e.g. -t tag1 -t tag2")]
        tags: Option<Vec<String>>,
        #[arg(short, long, help = "Mark the time entry as billable")]
        billable: bool,
        #[arg(
            long,
            help = "Start date/time. Accepted formats: RFC3339, YYYY-MM-DD HH:MM[:SS], YYYY-MM-DDTHH:MM[:SS], YYYY-MM-DD, HH:MM[:SS]"
        )]
        start: Option<String>,
        #[arg(
            long,
            help = "End date/time. Requires --start. Accepted formats: RFC3339, YYYY-MM-DD HH:MM[:SS], YYYY-MM-DDTHH:MM[:SS], YYYY-MM-DD, HH:MM[:SS]"
        )]
        end: Option<String>,
        #[arg(short, long, help = "Output in JSON format")]
        json: bool,
    },
    /// Continue a previous time entry.
    #[command(
        name = "continue",
        alias = "resume",
        after_long_help = "\
Examples:
  toggl entry continue              Continue most recent entry
  toggl entry continue --id 12345   Continue a specific entry by ID
  toggl entry continue --json       Continue and output as JSON"
    )]
    Resume {
        #[arg(short, long, help = "Continue a specific time entry by its ID")]
        id: Option<i64>,
        #[arg(short, long, help = "Output in JSON format")]
        json: bool,
    },
    /// Show details of a single time entry by ID or the currently running entry.
    #[command(after_long_help = "\
Examples:
  toggl entry show 12345
  toggl entry show 12345 --json
  toggl entry show --current
  toggl entry running        (alternative for running entry)")]
    Show {
        #[arg(help = "ID of the time entry to show")]
        id: Option<i64>,
        #[arg(short, long, help = "Show the currently running time entry")]
        current: bool,
        #[arg(short, long, help = "Output in JSON format")]
        json: bool,
    },
    /// Edit a time entry's description, billable state, project, task, or tags.
    #[command(
        name = "edit",
        alias = "update",
        after_long_help = "\
Examples:
  toggl entry edit --current -d \"New description\"
  toggl entry edit 12345 -p NewProject
  toggl entry edit 12345 --end \"\"     # Re-open a stopped entry
  toggl entry edit 12345 -p \"\"        # Remove project from entry"
    )]
    Update {
        #[arg(help = "ID of the time entry to edit")]
        id: Option<i64>,
        #[arg(short, long, help = "Edit the currently running time entry")]
        current: bool,
        #[arg(short, long, help = "New description")]
        description: Option<String>,
        #[arg(long, help = "New billable state (true/false)")]
        billable: Option<bool>,
        #[arg(
            short,
            long,
            help = "New project name (use empty string \"\" to remove project)"
        )]
        project: Option<String>,
        #[arg(long, help = "New task name (use empty string \"\" to remove task)")]
        task: Option<String>,
        #[arg(
            short,
            long,
            help = "Tag name (repeatable, use empty string \"\" to clear tags), e.g. -t tag1 -t tag2"
        )]
        tags: Option<Vec<String>>,
        #[arg(
            long,
            help = "New start date/time. Accepted formats: RFC3339, YYYY-MM-DD HH:MM[:SS], YYYY-MM-DDTHH:MM[:SS], YYYY-MM-DD, HH:MM[:SS]"
        )]
        start: Option<String>,
        #[arg(
            long,
            help = "New end date/time (use empty string \"\" to clear end time). Accepted formats: RFC3339, YYYY-MM-DD HH:MM[:SS], YYYY-MM-DDTHH:MM[:SS], YYYY-MM-DD, HH:MM[:SS]"
        )]
        end: Option<String>,
        #[arg(short, long, help = "Output in JSON format")]
        json: bool,
    },
    /// Delete a time entry by ID.
    #[command(after_long_help = "\
Examples:
  toggl entry delete 12345
  toggl entry delete --current
  toggl entry delete 12345 --json")]
    Delete {
        #[arg(help = "ID of the time entry to delete")]
        id: Option<i64>,
        #[arg(short, long, help = "Delete the currently running time entry")]
        current: bool,
        #[arg(short, long, help = "Output in JSON format")]
        json: bool,
    },
    /// Bulk edit multiple time entries with a JSON Patch payload.
    BulkEdit {
        #[arg(help = "IDs of the time entries to update")]
        ids: Vec<i64>,
        #[arg(long, help = "JSON Patch array to send to the bulk update endpoint")]
        json: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum ProjectAction {
    /// List projects.
    #[command(after_long_help = "\
Examples:
  toggl project list
  toggl project list --json | jq '.[].name'")]
    List {
        #[arg(short, long, help = "Output in JSON format")]
        json: bool,
    },
    /// Create a new project in your workspace.
    Create {
        #[arg(help = "Name of the project to create")]
        name: String,
        #[arg(
            short,
            long,
            help = "Hex color for the project (e.g. #06aaf5)",
            default_value = "#06aaf5"
        )]
        color: String,
    },
    /// Rename a project in your workspace.
    Rename {
        #[arg(help = "Current name of the project")]
        old_name: String,
        #[arg(help = "New name for the project")]
        new_name: String,
    },
    /// Delete a project from your workspace by name.
    Delete {
        #[arg(help = "Name of the project to delete")]
        name: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum TagAction {
    /// List tags in the current workspace.
    #[command(after_long_help = "\
Examples:
  toggl tag list
  toggl tag list --json")]
    List {
        #[arg(short, long, help = "Output in JSON format")]
        json: bool,
    },
    /// Create a new tag in your workspace.
    Create {
        #[arg(help = "Name of the tag to create")]
        name: String,
    },
    /// Rename a tag in your workspace.
    Rename {
        #[arg(help = "Current name of the tag")]
        old_name: String,
        #[arg(help = "New name for the tag")]
        new_name: String,
    },
    /// Delete a tag from your workspace by name.
    Delete {
        #[arg(help = "Name of the tag to delete")]
        name: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum ClientAction {
    /// List clients in the current workspace.
    #[command(after_long_help = "\
Examples:
  toggl client list
  toggl client list --json")]
    List {
        #[arg(short, long, help = "Output in JSON format")]
        json: bool,
    },
    /// Create a new client in your workspace.
    Create {
        #[arg(help = "Name of the client to create")]
        name: String,
    },
    /// Rename a client in your workspace.
    Rename {
        #[arg(help = "Current name of the client")]
        old_name: String,
        #[arg(help = "New name for the client")]
        new_name: String,
    },
    /// Delete a client from your workspace by name.
    Delete {
        #[arg(help = "Name of the client to delete")]
        name: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum TaskAction {
    /// List tasks.
    List {
        #[arg(short, long, help = "Output in JSON format")]
        json: bool,
    },
    /// Create a task inside a project.
    Create {
        #[arg(
            short,
            long,
            help = "Exact name of the project that should contain the task"
        )]
        project: String,
        #[arg(help = "Name of the task to create")]
        name: String,
        #[arg(long, help = "Task active state (true/false)")]
        active: Option<bool>,
        #[arg(long, help = "Estimated duration for the task in seconds")]
        estimated_seconds: Option<i64>,
        #[arg(long, help = "Assign the task to a specific user ID")]
        user_id: Option<i64>,
    },
    /// Update a task inside a project.
    Update {
        #[arg(short, long, help = "Exact name of the project that contains the task")]
        project: String,
        #[arg(help = "Current name of the task")]
        name: String,
        #[arg(long, help = "New name for the task")]
        new_name: Option<String>,
        #[arg(long, help = "Task active state (true/false)")]
        active: Option<bool>,
        #[arg(long, help = "Estimated duration for the task in seconds")]
        estimated_seconds: Option<i64>,
        #[arg(long, help = "Assign the task to a specific user ID")]
        user_id: Option<i64>,
    },
    /// Rename a task in a project.
    Rename {
        #[arg(short, long, help = "Exact name of the project that contains the task")]
        project: String,
        #[arg(help = "Current name of the task")]
        old_name: String,
        #[arg(help = "New name for the task")]
        new_name: String,
    },
    /// Delete a task from a project by name.
    Delete {
        #[arg(short, long, help = "Exact name of the project that contains the task")]
        project: String,
        #[arg(help = "Name of the task to delete")]
        name: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum WorkspaceAction {
    /// List workspaces.
    List {
        #[arg(short, long, help = "Output in JSON format")]
        json: bool,
    },
    /// Create a new workspace in an organization.
    Create {
        #[arg(help = "Organization ID that will own the workspace")]
        organization_id: i64,
        #[arg(help = "Name of the workspace to create")]
        name: String,
    },
    /// Rename one of your workspaces.
    Rename {
        #[arg(help = "Current name of the workspace")]
        old_name: String,
        #[arg(help = "New name for the workspace")]
        new_name: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum OrganizationAction {
    /// List organizations available to the current user.
    List {
        #[arg(short, long, help = "Output in JSON format")]
        json: bool,
    },
    /// Show one organization by ID.
    Show {
        #[arg(help = "Organization ID")]
        id: i64,
        #[arg(short, long, help = "Output in JSON format")]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum PreferencesAction {
    /// Show current user preferences.
    Read,
    /// Update current user preferences with a JSON payload.
    Update {
        #[arg(help = "JSON object to send to /me/preferences")]
        json: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Initialize a configuration file.
    Init,
    /// Report matching configuration block for current directory.
    Active,
}

#[derive(Subcommand, Debug)]
pub enum AuthAction {
    /// Login with your Toggl API token (interactive if no arguments provided).
    Login {
        #[arg(help = "API token for authentication (omit to enter interactively)")]
        api_token: Option<String>,
        #[arg(long, help = "Toggl service type: 'official' or 'opentoggl'")]
        api_type: Option<String>,
        #[arg(long, help = "API URL for self-hosted Toggl (required for opentoggl)")]
        api_url: Option<String>,
    },
    /// Show current authentication status, provider, and credential source.
    #[command(after_long_help = "\
Examples:
  toggl auth status               Show auth status
  toggl auth status --json        Output auth status as JSON")]
    Status {
        #[arg(short, long, help = "Output in JSON format")]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum ReportAction {
    /// Summary report grouped by project
    #[command(after_long_help = "\
Examples:
  toggl report summary
  toggl report summary --since today --until today
  toggl report summary --since yesterday --until today
  toggl report summary --since 2026-03-01 --until 2026-03-27 --json")]
    Summary {
        #[arg(
            long,
            help = "Start date (YYYY-MM-DD, or: today, yesterday, now, this_week, last_week). Default: this_week"
        )]
        since: Option<String>,
        #[arg(
            long,
            help = "End date (YYYY-MM-DD, or: today, yesterday, now, this_week, last_week). Default: today"
        )]
        until: Option<String>,
        #[arg(short, long, help = "Output in JSON format")]
        json: bool,
        #[arg(long, help = "Group by: projects, clients, users (default: projects)")]
        group_by: Option<String>,
        #[arg(long, help = "Sub-group by: time_entries, tasks, projects, users")]
        sub_group_by: Option<String>,
    },
    /// Detailed report listing individual time entries
    #[command(after_long_help = "\
Examples:
  toggl report detailed
  toggl report detailed --since today --until today
  toggl report detailed --since 2026-03-01 --until 2026-03-27 --json -n 100")]
    Detailed {
        #[arg(
            long,
            help = "Start date (YYYY-MM-DD, or: today, yesterday, now, this_week, last_week). Default: this_week"
        )]
        since: Option<String>,
        #[arg(
            long,
            help = "End date (YYYY-MM-DD, or: today, yesterday, now, this_week, last_week). Default: today"
        )]
        until: Option<String>,
        #[arg(short, long, help = "Output in JSON format")]
        json: bool,
        #[arg(short, long, help = "Maximum number of entries per page")]
        number: Option<i64>,
        #[arg(
            long,
            help = "Order by: date, user, duration, description (default: date)"
        )]
        order_by: Option<String>,
        #[arg(long, help = "Order direction: ASC or DESC")]
        order_dir: Option<String>,
    },
    /// Weekly report with daily breakdown
    #[command(after_long_help = "\
Examples:
  toggl report weekly
  toggl report weekly --since today --until today
  toggl report weekly --since 2026-03-17 --until 2026-03-23 --json")]
    Weekly {
        #[arg(
            long,
            help = "Start date (YYYY-MM-DD, or: today, yesterday, now, this_week, last_week). Default: this_week"
        )]
        since: Option<String>,
        #[arg(
            long,
            help = "End date (YYYY-MM-DD, or: today, yesterday, now, this_week, last_week). Default: today"
        )]
        until: Option<String>,
        #[arg(short, long, help = "Output in JSON format")]
        json: bool,
    },
}

impl Command {
    /// Returns true if the parsed command includes a `--json` output flag.
    /// Used by the top-level error handler to format errors as JSON when appropriate.
    pub fn has_json_flag(&self) -> bool {
        match self {
            Command::Me { json } => *json,
            Command::Entry { action } => match action {
                EntryAction::Current { json }
                | EntryAction::List { json, .. }
                | EntryAction::Stop { json }
                | EntryAction::Start { json, .. }
                | EntryAction::Resume { json, .. }
                | EntryAction::Show { json, .. }
                | EntryAction::Update { json, .. }
                | EntryAction::Delete { json, .. } => *json,
                EntryAction::BulkEdit { .. } => false,
            },
            Command::Project { action } => match action {
                ProjectAction::List { json } => *json,
                _ => false,
            },
            Command::Tag { action } => match action {
                TagAction::List { json } => *json,
                _ => false,
            },
            Command::Client { action } => match action {
                ClientAction::List { json } => *json,
                _ => false,
            },
            Command::Task { action } => match action {
                TaskAction::List { json } => *json,
                _ => false,
            },
            Command::Workspace { action } => match action {
                WorkspaceAction::List { json } => *json,
                _ => false,
            },
            Command::Org { action } => match action {
                OrganizationAction::List { json } | OrganizationAction::Show { json, .. } => *json,
            },
            Command::Auth { action, .. } => match action {
                Some(AuthAction::Status { json }) => *json,
                _ => false,
            },
            Command::Report { action } => match action {
                ReportAction::Summary { json, .. }
                | ReportAction::Detailed { json, .. }
                | ReportAction::Weekly { json, .. } => *json,
            },
            Command::Logout | Command::Preferences { .. } | Command::Config { .. } => false,
        }
    }
}

/// Entity types for list command (used internally by list.rs)
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Entity {
    Project { json: bool },
    TimeEntry { json: bool },
    Tag { json: bool },
    Client { json: bool },
    Workspace { json: bool },
    Task { json: bool },
    Organization { json: bool },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_parser_can_beconstructed() {
        // Verify the parser can be built - this tests the derive macros work
        let _ = Cli::command();
    }

    #[test]
    fn entry_subcommand_parses_without_id() {
        let cmd = Cli::try_parse_from(["toggl", "entry", "show"]).expect("should parse");
        match cmd.cmd {
            Command::Entry {
                action:
                    EntryAction::Show {
                        id: None,
                        current: false,
                        json: false,
                    },
            } => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn entry_show_with_id_parses() {
        let cmd = Cli::try_parse_from(["toggl", "entry", "show", "42"]).expect("should parse");
        match cmd.cmd {
            Command::Entry {
                action:
                    EntryAction::Show {
                        id: Some(42),
                        current: false,
                        json: false,
                    },
            } => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn entry_list_parses() {
        let cmd = Cli::try_parse_from(["toggl", "entry", "list"]).expect("should parse");
        match cmd.cmd {
            Command::Entry {
                action:
                    EntryAction::List {
                        number: None,
                        json: false,
                        since: None,
                        until: None,
                    },
            } => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn entry_start_parses() {
        let cmd = Cli::try_parse_from(["toggl", "entry", "start", "-d", "Test entry"])
            .expect("should parse");
        match cmd.cmd {
            Command::Entry {
                action:
                    EntryAction::Start {
                        description: Some(d),
                        ..
                    },
            } if d == "Test entry" => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn project_list_parses() {
        let cmd = Cli::try_parse_from(["toggl", "project", "list"]).expect("should parse");
        match cmd.cmd {
            Command::Project {
                action: ProjectAction::List { json: false },
            } => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn project_create_parses() {
        let cmd = Cli::try_parse_from(["toggl", "project", "create", "My Project"])
            .expect("should parse");
        match cmd.cmd {
            Command::Project {
                action: ProjectAction::Create { name, color },
            } if name == "My Project" && color == "#06aaf5" => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn tag_list_parses() {
        let cmd = Cli::try_parse_from(["toggl", "tag", "list"]).expect("should parse");
        match cmd.cmd {
            Command::Tag {
                action: TagAction::List { json: false },
            } => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn client_list_parses() {
        let cmd = Cli::try_parse_from(["toggl", "client", "list"]).expect("should parse");
        match cmd.cmd {
            Command::Client {
                action: ClientAction::List { json: false },
            } => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn project_list_json_parses() {
        let cmd =
            Cli::try_parse_from(["toggl", "project", "list", "--json"]).expect("should parse");
        match cmd.cmd {
            Command::Project {
                action: ProjectAction::List { json: true },
            } => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn project_rename_parses() {
        let cmd = Cli::try_parse_from(["toggl", "project", "rename", "OldName", "NewName"])
            .expect("should parse");
        match cmd.cmd {
            Command::Project {
                action: ProjectAction::Rename { old_name, new_name },
            } => {
                assert_eq!(old_name, "OldName");
                assert_eq!(new_name, "NewName");
            }
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn project_delete_parses() {
        let cmd =
            Cli::try_parse_from(["toggl", "project", "delete", "MyProject"]).expect("should parse");
        match cmd.cmd {
            Command::Project {
                action: ProjectAction::Delete { name },
            } => {
                assert_eq!(name, "MyProject");
            }
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn tag_list_json_parses() {
        let cmd = Cli::try_parse_from(["toggl", "tag", "list", "--json"]).expect("should parse");
        match cmd.cmd {
            Command::Tag {
                action: TagAction::List { json: true },
            } => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn tag_rename_parses() {
        let cmd = Cli::try_parse_from(["toggl", "tag", "rename", "OldTag", "NewTag"])
            .expect("should parse");
        match cmd.cmd {
            Command::Tag {
                action: TagAction::Rename { old_name, new_name },
            } => {
                assert_eq!(old_name, "OldTag");
                assert_eq!(new_name, "NewTag");
            }
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn tag_delete_parses() {
        let cmd = Cli::try_parse_from(["toggl", "tag", "delete", "MyTag"]).expect("should parse");
        match cmd.cmd {
            Command::Tag {
                action: TagAction::Delete { name },
            } => {
                assert_eq!(name, "MyTag");
            }
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn client_list_json_parses() {
        let cmd = Cli::try_parse_from(["toggl", "client", "list", "--json"]).expect("should parse");
        match cmd.cmd {
            Command::Client {
                action: ClientAction::List { json: true },
            } => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn client_rename_parses() {
        let cmd = Cli::try_parse_from(["toggl", "client", "rename", "OldClient", "NewClient"])
            .expect("should parse");
        match cmd.cmd {
            Command::Client {
                action: ClientAction::Rename { old_name, new_name },
            } => {
                assert_eq!(old_name, "OldClient");
                assert_eq!(new_name, "NewClient");
            }
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn client_delete_parses() {
        let cmd =
            Cli::try_parse_from(["toggl", "client", "delete", "MyClient"]).expect("should parse");
        match cmd.cmd {
            Command::Client {
                action: ClientAction::Delete { name },
            } => {
                assert_eq!(name, "MyClient");
            }
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn auth_command_allows_missing_token() {
        let cmd = Cli::try_parse_from(["toggl", "auth"])
            .expect("auth command should parse without a token");
        match cmd.cmd {
            Command::Auth {
                action: None,
                api_token: None,
                api_type: None,
                api_url: None,
            } => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn auth_login_subcommand_parses() {
        let cmd = Cli::try_parse_from(["toggl", "auth", "login", "my-token"])
            .expect("auth login should parse");
        match cmd.cmd {
            Command::Auth {
                action:
                    Some(AuthAction::Login {
                        api_token,
                        api_type: None,
                        api_url: None,
                    }),
                api_token: None,
                api_type: None,
                api_url: None,
            } => {
                assert_eq!(api_token, Some("my-token".to_string()));
            }
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn auth_status_subcommand_parses() {
        let cmd =
            Cli::try_parse_from(["toggl", "auth", "status"]).expect("auth status should parse");
        match cmd.cmd {
            Command::Auth {
                action: Some(AuthAction::Status { json: false }),
                api_token: None,
                api_type: None,
                api_url: None,
            } => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn config_init_parses() {
        let cmd = Cli::try_parse_from(["toggl", "config", "init"]).expect("should parse");
        match cmd.cmd {
            Command::Config {
                edit: false,
                delete: false,
                path: false,
                cmd: Some(ConfigAction::Init),
            } => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn config_manage_parses() {
        let cmd = Cli::try_parse_from(["toggl", "config"]).expect("should parse");
        match cmd.cmd {
            Command::Config {
                edit: false,
                delete: false,
                path: false,
                cmd: None,
            } => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn preferences_read_parses() {
        let cmd = Cli::try_parse_from(["toggl", "preferences", "read"]).expect("should parse");
        match cmd.cmd {
            Command::Preferences {
                action: PreferencesAction::Read,
            } => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn org_list_parses() {
        let cmd = Cli::try_parse_from(["toggl", "org", "list"]).expect("should parse");
        match cmd.cmd {
            Command::Org {
                action: OrganizationAction::List { json: false },
            } => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn workspace_list_parses() {
        let cmd = Cli::try_parse_from(["toggl", "workspace", "list"]).expect("should parse");
        match cmd.cmd {
            Command::Workspace {
                action: WorkspaceAction::List { json: false },
            } => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn task_list_parses() {
        let cmd = Cli::try_parse_from(["toggl", "task", "list"]).expect("should parse");
        match cmd.cmd {
            Command::Task {
                action: TaskAction::List { json: false },
            } => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn task_create_parses() {
        let cmd =
            Cli::try_parse_from(["toggl", "task", "create", "--project", "Platform", "Review"])
                .expect("should parse");
        match cmd.cmd {
            Command::Task {
                action:
                    TaskAction::Create {
                        project,
                        name,
                        active: None,
                        estimated_seconds: None,
                        user_id: None,
                    },
            } => {
                assert_eq!(project, "Platform");
                assert_eq!(name, "Review");
            }
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn task_update_parses() {
        let cmd = Cli::try_parse_from([
            "toggl",
            "task",
            "update",
            "--project",
            "Platform",
            "--new-name",
            "Review v2",
            "Review",
        ])
        .expect("should parse");
        match cmd.cmd {
            Command::Task {
                action:
                    TaskAction::Update {
                        project,
                        name,
                        new_name,
                        active: None,
                        estimated_seconds: None,
                        user_id: None,
                    },
            } => {
                assert_eq!(project, "Platform");
                assert_eq!(name, "Review");
                assert_eq!(new_name, Some("Review v2".to_string()));
            }
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn task_rename_parses() {
        let cmd = Cli::try_parse_from([
            "toggl",
            "task",
            "rename",
            "--project",
            "Platform",
            "OldName",
            "NewName",
        ])
        .expect("should parse");
        match cmd.cmd {
            Command::Task {
                action:
                    TaskAction::Rename {
                        project,
                        old_name,
                        new_name,
                    },
            } => {
                assert_eq!(project, "Platform");
                assert_eq!(old_name, "OldName");
                assert_eq!(new_name, "NewName");
            }
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn task_delete_parses() {
        let cmd =
            Cli::try_parse_from(["toggl", "task", "delete", "--project", "Platform", "Review"])
                .expect("should parse");
        match cmd.cmd {
            Command::Task {
                action: TaskAction::Delete { project, name },
            } => {
                assert_eq!(project, "Platform");
                assert_eq!(name, "Review");
            }
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn workspace_list_json_parses() {
        let cmd =
            Cli::try_parse_from(["toggl", "workspace", "list", "--json"]).expect("should parse");
        match cmd.cmd {
            Command::Workspace {
                action: WorkspaceAction::List { json: true },
            } => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn workspace_create_parses() {
        let cmd = Cli::try_parse_from(["toggl", "workspace", "create", "42", "Platform"])
            .expect("should parse");
        match cmd.cmd {
            Command::Workspace {
                action:
                    WorkspaceAction::Create {
                        organization_id,
                        name,
                    },
            } => {
                assert_eq!(organization_id, 42);
                assert_eq!(name, "Platform");
            }
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn workspace_rename_parses() {
        let cmd = Cli::try_parse_from(["toggl", "workspace", "rename", "OldName", "NewName"])
            .expect("should parse");
        match cmd.cmd {
            Command::Workspace {
                action: WorkspaceAction::Rename { old_name, new_name },
            } => {
                assert_eq!(old_name, "OldName");
                assert_eq!(new_name, "NewName");
            }
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn org_list_json_parses() {
        let cmd = Cli::try_parse_from(["toggl", "org", "list", "--json"]).expect("should parse");
        match cmd.cmd {
            Command::Org {
                action: OrganizationAction::List { json: true },
            } => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn org_show_parses() {
        let cmd = Cli::try_parse_from(["toggl", "org", "show", "42"]).expect("should parse");
        match cmd.cmd {
            Command::Org {
                action: OrganizationAction::Show { id, json: false },
            } => {
                assert_eq!(id, 42);
            }
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn org_show_json_parses() {
        let cmd =
            Cli::try_parse_from(["toggl", "org", "show", "42", "--json"]).expect("should parse");
        match cmd.cmd {
            Command::Org {
                action: OrganizationAction::Show { id, json: true },
            } => {
                assert_eq!(id, 42);
            }
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn preferences_update_parses() {
        let cmd = Cli::try_parse_from([
            "toggl",
            "preferences",
            "update",
            "{\"time_format\":\"H:mm\"}",
        ])
        .expect("should parse");
        match cmd.cmd {
            Command::Preferences {
                action: PreferencesAction::Update { json },
            } => {
                assert_eq!(json, "{\"time_format\":\"H:mm\"}");
            }
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn entry_current_parses() {
        let cmd = Cli::try_parse_from(["toggl", "entry", "current"]).expect("should parse");
        match cmd.cmd {
            Command::Entry {
                action: EntryAction::Current { json: false },
            } => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn entry_stop_parses() {
        let cmd = Cli::try_parse_from(["toggl", "entry", "stop"]).expect("should parse");
        match cmd.cmd {
            Command::Entry {
                action: EntryAction::Stop { json: false },
            } => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn entry_resume_parses() {
        let cmd = Cli::try_parse_from(["toggl", "entry", "resume"]).expect("should parse");
        match cmd.cmd {
            Command::Entry {
                action:
                    EntryAction::Resume {
                        id: None,
                        json: false,
                    },
            } => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn entry_delete_without_id_or_current_parses() {
        // delete without id or --current is now allowed at parse time
        // (validation happens at execution time)
        let result = Cli::try_parse_from(["toggl", "entry", "delete"]);
        // delete without args is now valid - requires either id or --current at runtime
        assert!(
            result.is_ok()
                && result
                    .map(|c| matches!(
                        c.cmd,
                        Command::Entry {
                            action: EntryAction::Delete {
                                id: None,
                                current: false,
                                json: false,
                            }
                        }
                    ))
                    .unwrap_or(false)
        );
    }

    #[test]
    fn entry_delete_with_current_flag_parses() {
        let cmd =
            Cli::try_parse_from(["toggl", "entry", "delete", "--current"]).expect("should parse");
        match cmd.cmd {
            Command::Entry {
                action:
                    EntryAction::Delete {
                        id: None,
                        current: true,
                        json: false,
                    },
            } => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn entry_update_without_id_or_current_parses() {
        // update without id or --current is allowed at parse time
        // (validation happens at execution time - must require one or the other)
        let result = Cli::try_parse_from(["toggl", "entry", "update"]);
        assert!(
            result.is_ok()
                && result
                    .map(|c| matches!(
                        c.cmd,
                        Command::Entry {
                            action: EntryAction::Update {
                                id: None,
                                current: false,
                                ..
                            }
                        }
                    ))
                    .unwrap_or(false)
        );
    }

    #[test]
    fn entry_update_with_id_parses() {
        let cmd = Cli::try_parse_from(["toggl", "entry", "update", "42"]).expect("should parse");
        match cmd.cmd {
            Command::Entry {
                action:
                    EntryAction::Update {
                        id: Some(42),
                        current: false,
                        ..
                    },
            } => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn entry_update_with_current_flag_parses() {
        let cmd =
            Cli::try_parse_from(["toggl", "entry", "update", "--current"]).expect("should parse");
        match cmd.cmd {
            Command::Entry {
                action:
                    EntryAction::Update {
                        id: None,
                        current: true,
                        ..
                    },
            } => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn entry_edit_alias_parses() {
        let cmd = Cli::try_parse_from(["toggl", "entry", "edit", "42", "-d", "Updated"])
            .expect("should parse");
        match cmd.cmd {
            Command::Entry {
                action:
                    EntryAction::Update {
                        id: Some(42),
                        current: false,
                        description: Some(d),
                        ..
                    },
            } if d == "Updated" => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn entry_edit_with_description_parses() {
        let cmd = Cli::try_parse_from(["toggl", "entry", "edit", "99", "-d", "New desc"])
            .expect("should parse");
        match cmd.cmd {
            Command::Entry {
                action:
                    EntryAction::Update {
                        id: Some(99),
                        description: Some(d),
                        project: None,
                        tags: None,
                        ..
                    },
            } if d == "New desc" => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn entry_edit_with_project_parses() {
        let cmd = Cli::try_parse_from(["toggl", "entry", "edit", "99", "-p", "MyProject"])
            .expect("should parse");
        match cmd.cmd {
            Command::Entry {
                action:
                    EntryAction::Update {
                        id: Some(99),
                        project: Some(p),
                        ..
                    },
            } if p == "MyProject" => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn entry_edit_with_tags_parses() {
        let cmd = Cli::try_parse_from([
            "toggl", "entry", "edit", "99", "--tags", "tag1", "--tags", "tag2", "--tags", "tag3",
        ])
        .expect("should parse");
        match cmd.cmd {
            Command::Entry {
                action:
                    EntryAction::Update {
                        id: Some(99),
                        tags: Some(t),
                        ..
                    },
            } if t == vec!["tag1", "tag2", "tag3"] => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn entry_edit_clear_end_parses() {
        // Passing --end "" should clear the end time (re-open a stopped entry)
        let cmd = Cli::try_parse_from(["toggl", "entry", "edit", "99", "--end", ""])
            .expect("should parse");
        match cmd.cmd {
            Command::Entry {
                action:
                    EntryAction::Update {
                        id: Some(99),
                        end: Some(e),
                        ..
                    },
            } if e.is_empty() => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn entry_edit_clear_project_parses() {
        // Passing -p "" should remove project from entry
        let cmd =
            Cli::try_parse_from(["toggl", "entry", "edit", "99", "-p", ""]).expect("should parse");
        match cmd.cmd {
            Command::Entry {
                action:
                    EntryAction::Update {
                        id: Some(99),
                        project: Some(p),
                        ..
                    },
            } if p.is_empty() => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn entry_edit_with_current_and_description_parses() {
        let cmd =
            Cli::try_parse_from(["toggl", "entry", "edit", "--current", "-d", "Working on it"])
                .expect("should parse");
        match cmd.cmd {
            Command::Entry {
                action:
                    EntryAction::Update {
                        id: None,
                        current: true,
                        description: Some(d),
                        ..
                    },
            } if d == "Working on it" => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn entry_edit_multiple_fields_parses() {
        let cmd = Cli::try_parse_from([
            "toggl",
            "entry",
            "edit",
            "42",
            "-d",
            "New desc",
            "-p",
            "MyProject",
            "--tags",
            "tag1",
            "--tags",
            "tag2",
            "--billable",
            "true",
        ])
        .expect("should parse");
        match cmd.cmd {
            Command::Entry {
                action:
                    EntryAction::Update {
                        id: Some(42),
                        description: Some(d),
                        project: Some(p),
                        tags: Some(t),
                        billable: Some(true),
                        ..
                    },
            } if d == "New desc" && p == "MyProject" && t == vec!["tag1", "tag2"] => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn entry_edit_json_output_parses() {
        let cmd = Cli::try_parse_from(["toggl", "entry", "edit", "42", "-d", "Test", "--json"])
            .expect("should parse");
        match cmd.cmd {
            Command::Entry {
                action:
                    EntryAction::Update {
                        id: Some(42),
                        json: true,
                        description: Some(d),
                        ..
                    },
            } if d == "Test" => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn entry_bulk_edit_requires_ids() {
        let cmd = Cli::try_parse_from(["toggl", "entry", "bulk-edit", "1", "2", "3"])
            .expect("should parse");
        match cmd.cmd {
            Command::Entry {
                action: EntryAction::BulkEdit { ids, json: None },
            } => {
                assert_eq!(ids, vec![1, 2, 3]);
            }
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn entry_bulk_edit_parses_json_flag() {
        let payload = r#"[{"op":"replace","path":"/description","value":"test"}]"#;
        let cmd = Cli::try_parse_from(["toggl", "entry", "bulk-edit", "1", "2", "--json", payload])
            .expect("should parse");
        match cmd.cmd {
            Command::Entry {
                action: EntryAction::BulkEdit { ids, json: Some(j) },
            } => {
                assert_eq!(ids, vec![1, 2]);
                assert_eq!(j, payload);
            }
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn entry_bulk_edit_no_ids_parses_empty() {
        let cmd = Cli::try_parse_from(["toggl", "entry", "bulk-edit"]).expect("should parse");
        match cmd.cmd {
            Command::Entry {
                action: EntryAction::BulkEdit { ids, json: None },
            } => {
                assert!(ids.is_empty());
            }
            other => panic!("unexpected parse result: {other:?}"),
        }
    }
}
