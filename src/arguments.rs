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

    #[structopt(long, help = "Use fzf instead of the default picker")]
    pub fzf: bool,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    Current,
    #[structopt()]
    List {
        #[structopt(short, long)]
        number: Option<usize>,
        #[structopt(short, long, help = "Output in JSON format")]
        json: bool,
        #[structopt(
            long,
            help = "Filter entries starting on or after this date (YYYY-MM-DD)"
        )]
        since: Option<String>,
        #[structopt(
            long,
            help = "Filter entries starting on or before this date (YYYY-MM-DD)"
        )]
        until: Option<String>,
        #[structopt(subcommand)]
        entity: Option<Entity>,
    },
    Running,
    Stop,
    #[structopt(
        about = "Authenticate with the Toggl API. Find your API token at https://track.toggl.com/profile#api-token"
    )]
    Auth {
        api_token: String,
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
    Continue {
        #[structopt(short, long)]
        interactive: bool,
    },
    #[structopt(about = "Edit a time entry's description, billable state, project, task, or tags")]
    Edit {
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
    #[structopt(about = "Delete a time entry by ID")]
    Delete {
        #[structopt(help = "ID of the time entry to delete")]
        id: i64,
    },
    #[structopt(about = "Bulk edit multiple time entries with a JSON Patch payload")]
    BulkEditTimeEntries {
        #[structopt(help = "IDs of the time entries to update")]
        ids: Vec<i64>,
        #[structopt(long, help = "JSON Patch array to send to the bulk update endpoint")]
        json: String,
    },
    #[structopt(about = "Create a new project in your workspace")]
    CreateProject {
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
    #[structopt(about = "Delete a project from your workspace by name")]
    DeleteProject {
        #[structopt(help = "Name of the project to delete")]
        name: String,
    },
    #[structopt(about = "Rename a project in your workspace")]
    RenameProject {
        #[structopt(help = "Current name of the project")]
        old_name: String,
        #[structopt(help = "New name for the project")]
        new_name: String,
    },
    #[structopt(about = "Create a new tag in your workspace")]
    CreateTag {
        #[structopt(help = "Name of the tag to create")]
        name: String,
    },
    #[structopt(about = "Delete a tag from your workspace by name")]
    DeleteTag {
        #[structopt(help = "Name of the tag to delete")]
        name: String,
    },
    #[structopt(about = "Rename a tag in your workspace")]
    RenameTag {
        #[structopt(help = "Current name of the tag")]
        old_name: String,
        #[structopt(help = "New name for the tag")]
        new_name: String,
    },
    #[structopt(about = "Show details of a single time entry by ID")]
    Show {
        #[structopt(help = "ID of the time entry to show")]
        id: i64,
        #[structopt(short, long, help = "Output in JSON format")]
        json: bool,
    },
    #[structopt(about = "Show current user profile information")]
    Me,
    #[structopt(about = "Show current user preferences")]
    Preferences,
    #[structopt(about = "Update current user preferences with a JSON payload")]
    UpdatePreferences {
        #[structopt(help = "JSON object to send to /me/preferences")]
        json: String,
    },
    #[structopt(about = "Create a new client in your workspace")]
    CreateClient {
        #[structopt(help = "Name of the client to create")]
        name: String,
    },
    #[structopt(about = "Delete a client from your workspace by name")]
    DeleteClient {
        #[structopt(help = "Name of the client to delete")]
        name: String,
    },
    #[structopt(about = "Rename a client in your workspace")]
    RenameClient {
        #[structopt(help = "Current name of the client")]
        old_name: String,
        #[structopt(help = "New name for the client")]
        new_name: String,
    },
    #[structopt(about = "Create a new workspace in an organization")]
    CreateWorkspace {
        #[structopt(help = "Organization ID that will own the workspace")]
        organization_id: i64,
        #[structopt(help = "Name of the workspace to create")]
        name: String,
    },
    #[structopt(about = "Rename one of your workspaces")]
    RenameWorkspace {
        #[structopt(help = "Current name of the workspace")]
        old_name: String,
        #[structopt(help = "New name for the workspace")]
        new_name: String,
    },
    #[structopt(about = "Create a task inside a project")]
    CreateTask {
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
    #[structopt(about = "Update a task inside a project")]
    UpdateTask {
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
    #[structopt(about = "Delete a task from a project by name")]
    DeleteTask {
        #[structopt(short, long, help = "Exact name of the project that contains the task")]
        project: String,
        #[structopt(help = "Name of the task to delete")]
        name: String,
    },
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
pub enum Entity {
    Project {
        #[structopt(short, long, help = "Output in JSON format")]
        json: bool,
    },
    TimeEntry {
        #[structopt(short, long, help = "Output in JSON format")]
        json: bool,
    },
    Tag {
        #[structopt(short, long, help = "Output in JSON format")]
        json: bool,
    },
    Client {
        #[structopt(short, long, help = "Output in JSON format")]
        json: bool,
    },
    Workspace {
        #[structopt(short, long, help = "Output in JSON format")]
        json: bool,
    },
    Task {
        #[structopt(short, long, help = "Output in JSON format")]
        json: bool,
    },
}
#[derive(Debug, StructOpt)]
pub enum ConfigSubCommand {
    #[structopt(about = "Initialize a configuration file.")]
    Init,
    #[structopt(about = "Report matching configuration block for current directory.")]
    Active,
}
