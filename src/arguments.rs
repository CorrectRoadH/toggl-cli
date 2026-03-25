use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// Toggl command line app.
#[derive(Parser, Debug)]
#[command(name = "toggl")]
#[command(about = "Toggl command line app.", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Command,

    #[arg(short = 'C', help = "Change directory before running the command")]
    pub directory: Option<PathBuf>,

    #[arg(long, help = "Use custom proxy")]
    pub proxy: Option<String>,

    #[arg(
        long,
        help = "Use fzf for interactive selections instead of the default picker"
    )]
    pub fzf: bool,
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
    Me,
    /// Manage time entries.
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
    /// Show the current time entry.
    Current,
    /// List time entries.
    List {
        #[arg(short, long, help = "Maximum number of items to print")]
        number: Option<usize>,
        #[arg(long, help = "Maximum number of items to print (alias for --number)")]
        limit: Option<usize>,
        #[arg(short, long, help = "Output in JSON format")]
        json: bool,
        #[arg(
            long,
            help = "Filter time entries starting on or after this date/time; date-only values use local 00:00:00"
        )]
        since: Option<String>,
        #[arg(
            long,
            help = "Filter time entries before this date/time; date-only values include the entire local day"
        )]
        until: Option<String>,
    },
    /// Stop the currently running time entry.
    Stop,
    /// Start a new time entry, call with no arguments to start in interactive mode.
    Start {
        #[arg(short, long)]
        interactive: bool,
        #[arg(short, help = "Description of the time entry")]
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
        #[arg(
            short,
            long,
            help = "Space separated list of tags to associate with the time entry, e.g. 'tag1 tag2 tag3'"
        )]
        tags: Option<Vec<String>>,
        #[arg(short, long)]
        billable: bool,
        #[arg(
            long,
            help = "Start date/time. Accepted formats: RFC3339, YYYY-MM-DD HH:MM[:SS], YYYY-MM-DDTHH:MM[:SS], YYYY-MM-DD"
        )]
        start: Option<String>,
        #[arg(
            long,
            help = "End date/time. Requires --start. Accepted formats: RFC3339, YYYY-MM-DD HH:MM[:SS], YYYY-MM-DDTHH:MM[:SS], YYYY-MM-DD"
        )]
        end: Option<String>,
    },
    /// Continue a previous time entry.
    Resume {
        #[arg(short, long)]
        interactive: bool,
    },
    /// Show details of a single time entry by ID.
    Show {
        #[arg(help = "ID of the time entry to show")]
        id: Option<i64>,
        #[arg(short, long, help = "Output in JSON format")]
        json: bool,
    },
    /// Edit a time entry's description, billable state, project, task, or tags.
    Update {
        #[arg(help = "ID of the time entry to edit")]
        id: Option<i64>,
        #[arg(long, help = "Edit the currently running time entry")]
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
            help = "New space-separated list of tags (use empty string \"\" to clear tags)"
        )]
        tags: Option<Vec<String>>,
        #[arg(
            long,
            help = "New start date/time. Accepted formats: RFC3339, YYYY-MM-DD HH:MM[:SS], YYYY-MM-DDTHH:MM[:SS], YYYY-MM-DD"
        )]
        start: Option<String>,
        #[arg(
            long,
            help = "New end date/time (use empty string \"\" to clear end time). Accepted formats: RFC3339, YYYY-MM-DD HH:MM[:SS], YYYY-MM-DDTHH:MM[:SS], YYYY-MM-DD"
        )]
        end: Option<String>,
    },
    /// Delete a time entry by ID.
    Delete {
        #[arg(help = "ID of the time entry to delete")]
        id: Option<i64>,
        #[arg(long, help = "Delete the currently running time entry")]
        current: bool,
    },
    /// Bulk edit multiple time entries with a JSON Patch payload.
    #[clap(hide = true)]
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
    Status,
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
                        limit: None,
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
                action: Some(AuthAction::Status),
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
    fn entry_current_parses() {
        let cmd = Cli::try_parse_from(["toggl", "entry", "current"]).expect("should parse");
        match cmd.cmd {
            Command::Entry {
                action: EntryAction::Current,
            } => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn entry_stop_parses() {
        let cmd = Cli::try_parse_from(["toggl", "entry", "stop"]).expect("should parse");
        match cmd.cmd {
            Command::Entry {
                action: EntryAction::Stop,
            } => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn entry_resume_parses() {
        let cmd = Cli::try_parse_from(["toggl", "entry", "resume"]).expect("should parse");
        match cmd.cmd {
            Command::Entry {
                action: EntryAction::Resume { interactive: false },
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
                                current: false
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
}
