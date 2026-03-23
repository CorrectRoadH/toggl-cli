use std::path::PathBuf;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "toggl", about = "Toggl command line app.")]
pub struct CommandLineArguments {
    #[structopt(subcommand)]
    pub cmd: Option<Command>,

    #[structopt(short = "C", help = "Change directory before running the command")]
    pub directory: Option<PathBuf>,

    #[structopt(long, help = "Use custom proxy")]
    pub proxy: Option<String>,

    #[structopt(
        long,
        help = "Use fzf for interactive selections instead of the default picker"
    )]
    pub fzf: bool,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    #[structopt(about = "Show the current time entry")]
    Current,
    #[structopt(
        about = "List time entries or workspace resources",
        long_about = "List time entries by default, or list projects, tags, clients, workspaces, or tasks via a subcommand.\n\nDate-only values for --since/--until are interpreted in local time. --since uses the start of the given day, and --until includes the entire given day.\n\nExamples:\n  toggl list\n  toggl list --since 2026-03-01 --until 2026-03-06\n  toggl list --since 2026-03-06 --until 2026-03-06\n  toggl list project\n  toggl list tag --json"
    )]
    List {
        #[structopt(short, long, help = "Maximum number of items to print")]
        number: Option<usize>,
        #[structopt(
            short,
            long,
            help = "Output in JSON format (applies to the default time-entry listing)"
        )]
        json: bool,
        #[structopt(
            long,
            help = "Filter time entries starting on or after this date/time; date-only values use local 00:00:00"
        )]
        since: Option<String>,
        #[structopt(
            long,
            help = "Filter time entries before this date/time; date-only values include the entire local day"
        )]
        until: Option<String>,
        #[structopt(subcommand)]
        entity: Option<Entity>,
    },
    #[structopt(about = "Show the currently running time entry")]
    Running,
    #[structopt(about = "Stop the currently running time entry")]
    Stop,
    #[structopt(
        about = "Authenticate with the Toggl API. Find your API token at https://track.toggl.com/profile#api-token"
    )]
    Auth {
        api_token: Option<String>,
        #[structopt(long, help = "Toggl service type: 'official' or 'opentoggl'")]
        api_type: Option<String>,
        #[structopt(
            long,
            help = "Custom API base URL (e.g. https://your-instance.com/api/v9)"
        )]
        api_url: Option<String>,
    },
    #[structopt(about = "Clear stored credentials")]
    Logout,
    #[structopt(
        about = "Start a new time entry, call with no arguments to start in interactive mode"
    )]
    Start {
        #[structopt(short, long)]
        interactive: bool,
        #[structopt(help = "Description of the time entry")]
        description: Option<String>,
        #[structopt(
            short,
            long,
            help = "Exact name of the project you want the time entry to be associated with"
        )]
        project: Option<String>,
        #[structopt(
            long,
            help = "Exact name of the task you want the time entry to be associated with"
        )]
        task: Option<String>,
        #[structopt(
            short,
            long,
            help = "Space separated list of tags to associate with the time entry, e.g. 'tag1 tag2 tag3'"
        )]
        tags: Option<Vec<String>>,
        #[structopt(short, long)]
        billable: bool,
        #[structopt(
            long,
            help = "Start date/time. Accepted formats: RFC3339, YYYY-MM-DD HH:MM[:SS], YYYY-MM-DDTHH:MM[:SS], YYYY-MM-DD"
        )]
        start: Option<String>,
        #[structopt(
            long,
            help = "End date/time. Requires --start. Accepted formats: RFC3339, YYYY-MM-DD HH:MM[:SS], YYYY-MM-DDTHH:MM[:SS], YYYY-MM-DD"
        )]
        end: Option<String>,
    },
    #[structopt(about = "Continue a previous time entry")]
    Continue {
        #[structopt(short, long)]
        interactive: bool,
    },
    #[structopt(about = "Edit a resource (time entry, task, or preferences)")]
    Edit {
        #[structopt(subcommand)]
        entity: EditEntity,
    },
    #[structopt(about = "Delete a resource or a time entry by ID")]
    Delete {
        #[structopt(subcommand)]
        entity: Option<DeleteEntity>,
        #[structopt(help = "ID of the time entry to delete")]
        id: Option<i64>,
    },
    #[structopt(about = "Bulk edit multiple time entries with a JSON Patch payload")]
    BulkEditTimeEntries {
        #[structopt(help = "IDs of the time entries to update")]
        ids: Vec<i64>,
        #[structopt(long, help = "JSON Patch array to send to the bulk update endpoint")]
        json: Option<String>,
    },
    #[structopt(about = "Create a new resource in your workspace")]
    Create {
        #[structopt(subcommand)]
        entity: CreateEntity,
    },
    #[structopt(about = "Rename a resource in your workspace")]
    Rename {
        #[structopt(subcommand)]
        entity: RenameEntity,
    },
    #[structopt(about = "Show details of a single time entry by ID")]
    Show {
        #[structopt(help = "ID of the time entry to show")]
        id: Option<i64>,
        #[structopt(short, long, help = "Output in JSON format")]
        json: bool,
    },
    #[structopt(about = "Show current user profile information")]
    Me,
    #[structopt(about = "Inspect organizations available to the current user")]
    Organization {
        #[structopt(subcommand)]
        entity: Option<OrganizationEntity>,
    },
    #[structopt(about = "Show current user preferences")]
    Preferences,
    #[structopt(about = "Manage auto-tracking configuration")]
    Config {
        #[structopt(
            short,
            long,
            help = "Edit the configuration file in $EDITOR, defaults to vim"
        )]
        edit: bool,

        #[structopt(short, long, help = "Delete the configuration file")]
        delete: bool,

        #[structopt(short, long, help = "Print the path of the configuration file")]
        path: bool,

        #[structopt(subcommand)]
        cmd: Option<ConfigSubCommand>,
    },
}

#[derive(Debug, StructOpt)]
pub enum CreateEntity {
    #[structopt(about = "Create a new project in your workspace")]
    Project {
        #[structopt(help = "Name of the project to create")]
        name: String,
        #[structopt(
            short,
            long,
            help = "Hex color for the project (e.g. #06aaf5)",
            default_value = "#06aaf5"
        )]
        color: String,
    },
    #[structopt(about = "Create a new tag in your workspace")]
    Tag {
        #[structopt(help = "Name of the tag to create")]
        name: String,
    },
    #[structopt(about = "Create a new client in your workspace")]
    Client {
        #[structopt(help = "Name of the client to create")]
        name: String,
    },
    #[structopt(about = "Create a new workspace in an organization")]
    Workspace {
        #[structopt(help = "Organization ID that will own the workspace")]
        organization_id: i64,
        #[structopt(help = "Name of the workspace to create")]
        name: String,
    },
    #[structopt(about = "Create a task inside a project")]
    Task {
        #[structopt(
            short,
            long,
            help = "Exact name of the project that should contain the task"
        )]
        project: String,
        #[structopt(help = "Name of the task to create")]
        name: String,
        #[structopt(long, help = "Task active state (true/false)")]
        active: Option<bool>,
        #[structopt(long, help = "Estimated duration for the task in seconds")]
        estimated_seconds: Option<i64>,
        #[structopt(long, help = "Assign the task to a specific user ID")]
        user_id: Option<i64>,
    },
}

#[derive(Debug, StructOpt)]
pub enum DeleteEntity {
    #[structopt(about = "Delete a project from your workspace by name")]
    Project {
        #[structopt(help = "Name of the project to delete")]
        name: String,
    },
    #[structopt(about = "Delete a tag from your workspace by name")]
    Tag {
        #[structopt(help = "Name of the tag to delete")]
        name: String,
    },
    #[structopt(about = "Delete a client from your workspace by name")]
    Client {
        #[structopt(help = "Name of the client to delete")]
        name: String,
    },
    #[structopt(about = "Delete a task from a project by name")]
    Task {
        #[structopt(short, long, help = "Exact name of the project that contains the task")]
        project: String,
        #[structopt(help = "Name of the task to delete")]
        name: String,
    },
}

#[derive(Debug, StructOpt)]
pub enum EditEntity {
    #[structopt(about = "Edit a time entry's description, billable state, project, task, or tags")]
    TimeEntry {
        #[structopt(
            help = "ID of the time entry to edit (omit to edit the currently running entry)"
        )]
        id: Option<i64>,
        #[structopt(short, long, help = "New description")]
        description: Option<String>,
        #[structopt(long, help = "New billable state (true/false)")]
        billable: Option<bool>,
        #[structopt(
            short,
            long,
            help = "New project name (use empty string \"\" to remove project)"
        )]
        project: Option<String>,
        #[structopt(long, help = "New task name (use empty string \"\" to remove task)")]
        task: Option<String>,
        #[structopt(
            short,
            long,
            help = "New space-separated list of tags (use empty string \"\" to clear tags)"
        )]
        tags: Option<Vec<String>>,
        #[structopt(
            long,
            help = "New start date/time. Accepted formats: RFC3339, YYYY-MM-DD HH:MM[:SS], YYYY-MM-DDTHH:MM[:SS], YYYY-MM-DD"
        )]
        start: Option<String>,
        #[structopt(
            long,
            help = "New end date/time (use empty string \"\" to clear end time). Accepted formats: RFC3339, YYYY-MM-DD HH:MM[:SS], YYYY-MM-DDTHH:MM[:SS], YYYY-MM-DD"
        )]
        end: Option<String>,
    },
    #[structopt(about = "Update a task inside a project")]
    Task {
        #[structopt(short, long, help = "Exact name of the project that contains the task")]
        project: String,
        #[structopt(help = "Current name of the task")]
        name: String,
        #[structopt(long, help = "New name for the task")]
        new_name: Option<String>,
        #[structopt(long, help = "Task active state (true/false)")]
        active: Option<bool>,
        #[structopt(long, help = "Estimated duration for the task in seconds")]
        estimated_seconds: Option<i64>,
        #[structopt(long, help = "Assign the task to a specific user ID")]
        user_id: Option<i64>,
    },
    #[structopt(about = "Update current user preferences with a JSON payload")]
    Preferences {
        #[structopt(help = "JSON object to send to /me/preferences")]
        json: String,
    },
}

#[derive(Debug, StructOpt)]
pub enum RenameEntity {
    #[structopt(about = "Rename a project in your workspace")]
    Project {
        #[structopt(help = "Current name of the project")]
        old_name: String,
        #[structopt(help = "New name for the project")]
        new_name: String,
    },
    #[structopt(about = "Rename a tag in your workspace")]
    Tag {
        #[structopt(help = "Current name of the tag")]
        old_name: String,
        #[structopt(help = "New name for the tag")]
        new_name: String,
    },
    #[structopt(about = "Rename a client in your workspace")]
    Client {
        #[structopt(help = "Current name of the client")]
        old_name: String,
        #[structopt(help = "New name for the client")]
        new_name: String,
    },
    #[structopt(about = "Rename one of your workspaces")]
    Workspace {
        #[structopt(help = "Current name of the workspace")]
        old_name: String,
        #[structopt(help = "New name for the workspace")]
        new_name: String,
    },
}

#[derive(Debug, StructOpt)]
pub enum Entity {
    #[structopt(about = "List projects")]
    Project {
        #[structopt(short, long, help = "Output in JSON format")]
        json: bool,
    },
    #[structopt(about = "List time entries")]
    TimeEntry {
        #[structopt(short, long, help = "Output in JSON format")]
        json: bool,
    },
    #[structopt(about = "List tags in the current workspace")]
    Tag {
        #[structopt(short, long, help = "Output in JSON format")]
        json: bool,
    },
    #[structopt(about = "List clients in the current workspace")]
    Client {
        #[structopt(short, long, help = "Output in JSON format")]
        json: bool,
    },
    #[structopt(about = "List workspaces")]
    Workspace {
        #[structopt(short, long, help = "Output in JSON format")]
        json: bool,
    },
    #[structopt(about = "List tasks")]
    Task {
        #[structopt(short, long, help = "Output in JSON format")]
        json: bool,
    },
    #[structopt(about = "List organizations available to the current user")]
    Organization {
        #[structopt(short, long, help = "Output in JSON format")]
        json: bool,
    },
}

#[derive(Debug, StructOpt)]
pub enum OrganizationEntity {
    #[structopt(about = "List organizations available to the current user")]
    List {
        #[structopt(short, long, help = "Output in JSON format")]
        json: bool,
    },
    #[structopt(about = "Show one organization by ID")]
    Show {
        #[structopt(help = "Organization ID")]
        id: i64,
        #[structopt(short, long, help = "Output in JSON format")]
        json: bool,
    },
}

#[cfg(test)]
mod tests {
    use super::{Command, CommandLineArguments};
    use structopt::StructOpt;

    #[test]
    fn organization_command_allows_missing_nested_subcommand() {
        let parsed = CommandLineArguments::from_iter_safe(["toggl", "organization"])
            .expect("organization command should parse without a nested subcommand");

        match parsed.cmd {
            Some(Command::Organization { entity: None }) => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn auth_command_allows_missing_token() {
        let parsed = CommandLineArguments::from_iter_safe(["toggl", "auth"])
            .expect("auth command should parse without a token");

        match parsed.cmd {
            Some(Command::Auth {
                api_token: None,
                api_type: _,
                api_url: _,
            }) => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn show_command_allows_missing_id() {
        let parsed = CommandLineArguments::from_iter_safe(["toggl", "show"])
            .expect("show command should parse without an id");

        match parsed.cmd {
            Some(Command::Show {
                id: None,
                json: false,
            }) => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn bulk_edit_command_allows_missing_json() {
        let parsed = CommandLineArguments::from_iter_safe(["toggl", "bulk-edit-time-entries"])
            .expect("bulk edit command should parse without json");

        match parsed.cmd {
            Some(Command::BulkEditTimeEntries { ids, json: None }) if ids.is_empty() => {}
            other => panic!("unexpected parse result: {other:?}"),
        }
    }
}
#[derive(Debug, StructOpt)]
pub enum ConfigSubCommand {
    #[structopt(about = "Initialize a configuration file.")]
    Init,
    #[structopt(about = "Report matching configuration block for current directory.")]
    Active,
}
