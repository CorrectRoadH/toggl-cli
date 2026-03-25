mod api;
mod arguments;
mod commands;
mod config;
mod constants;
mod credentials;
mod error;
mod models;
mod picker;
mod utilities;

use utilities::read_from_stdin_with_constraints;

use api::client::ApiClient;
use api::client::V9ApiClient;
use arguments::Cli;
use arguments::{
    ClientAction, Command, ConfigAction, EntryAction, OrganizationAction, PreferencesAction,
    ProjectAction, TagAction, TaskAction, WorkspaceAction,
};
use clap::Parser;
use commands::auth::AuthenticationCommand;
use commands::bulk_edit_time_entries::BulkEditTimeEntriesCommand;
use commands::cont::ContinueCommand;
use commands::create_client::CreateClientCommand;
use commands::create_project::CreateProjectCommand;
use commands::create_tag::CreateTagCommand;
use commands::create_task::CreateTaskCommand;
use commands::create_workspace::CreateWorkspaceCommand;
use commands::delete::DeleteCommand;
use commands::delete_client::DeleteClientCommand;
use commands::delete_project::DeleteProjectCommand;
use commands::delete_tag::DeleteTagCommand;
use commands::delete_task::DeleteTaskCommand;
use commands::edit::EditCommand;
use commands::list::ListCommand;
use commands::me::MeCommand;
use commands::organization::OrganizationCommand;
use commands::preferences::PreferencesCommand;
use commands::rename_client::RenameClientCommand;
use commands::rename_project::RenameProjectCommand;
use commands::rename_tag::RenameTagCommand;
use commands::rename_workspace::RenameWorkspaceCommand;
use commands::running::RunningTimeEntryCommand;
use commands::show::ShowCommand;
use commands::start::StartCommand;
use commands::stop::{StopCommand, StopCommandOrigin};
use commands::update_preferences::UpdatePreferencesCommand;
use commands::update_task::UpdateTaskCommand;
use credentials::get_storage;
use credentials::Credentials;
use models::ResultWithDefaultError;
use once_cell::sync::OnceCell;
use std::io::{self};

static CACHED_CREDENTIALS: OnceCell<Credentials> = OnceCell::new();

#[tokio::main]
async fn main() -> ResultWithDefaultError<()> {
    let parsed_args = Cli::parse();
    match execute_subcommand(parsed_args).await {
        Ok(()) => Ok(()),
        Err(error) => {
            eprint!("{error}");
            std::process::exit(1);
        }
    }
}

async fn execute_subcommand(args: Cli) -> ResultWithDefaultError<()> {
    setup_working_directory(args.directory)?;

    let picker = picker::get_picker(args.fzf);

    match args.cmd {
        Command::Auth {
            api_token,
            api_type,
            api_url,
        } => execute_auth_command(api_token, api_type, api_url, args.proxy).await,
        Command::Logout => execute_logout_command().await,
        Command::Me => {
            let api_client = get_api_client(args.proxy.clone())?;
            MeCommand::execute(api_client).await
        }
        Command::Entry { action } => execute_entry_command(action, args.proxy, picker).await,
        Command::Project { action } => {
            let api_client = get_api_client(args.proxy.clone())?;
            execute_project_command(action, api_client).await
        }
        Command::Tag { action } => {
            let api_client = get_api_client(args.proxy.clone())?;
            execute_tag_command(action, api_client).await
        }
        Command::Client { action } => {
            let api_client = get_api_client(args.proxy.clone())?;
            execute_client_command(action, api_client).await
        }
        Command::Task { action } => {
            let api_client = get_api_client(args.proxy.clone())?;
            execute_task_command(action, api_client).await
        }
        Command::Workspace { action } => {
            let api_client = get_api_client(args.proxy.clone())?;
            execute_workspace_command(action, api_client).await
        }
        Command::Org { action } => {
            let api_client = get_api_client(args.proxy.clone())?;
            execute_organization_command(action, api_client).await
        }
        Command::Preferences { action } => {
            let api_client = get_api_client(args.proxy.clone())?;
            execute_preferences_command(action, api_client).await
        }
        Command::Config {
            edit,
            delete,
            path,
            cmd,
        } => execute_config_command(edit, delete, path, cmd).await,
    }
}

fn setup_working_directory(directory: Option<std::path::PathBuf>) -> ResultWithDefaultError<()> {
    if let Some(dir) = directory {
        if !dir.exists() {
            return Err(Box::new(error::ArgumentError::DirectoryNotFound(dir)));
        }
        if !dir.is_dir() {
            return Err(Box::new(error::ArgumentError::NotADirectory(dir)));
        }
        std::env::set_current_dir(dir).expect("Couldn't set current directory");
    }
    Ok(())
}

async fn execute_entry_command(
    action: EntryAction,
    proxy: Option<String>,
    picker: Box<dyn picker::ItemPicker>,
) -> ResultWithDefaultError<()> {
    // Validate required arguments BEFORE creating the API client to avoid
    // unnecessary keychain/storage access for validation failures.
    match &action {
        EntryAction::Show { id, .. } if id.is_none() => {
            eprintln!("error: 'show' requires an entry ID");
            return Err(Box::new(error::ArgumentError::ResourceNotFound(
                "entry ID is required".to_string(),
            )));
        }
        EntryAction::Delete { id } if id.is_none() => {
            eprintln!("error: 'delete' requires an entry ID");
            return Err(Box::new(error::ArgumentError::ResourceNotFound(
                "entry ID is required".to_string(),
            )));
        }
        EntryAction::BulkEdit { json: None, .. } => {
            eprintln!("error: 'bulk-edit' requires --json flag with JSON payload");
            return Err(Box::new(error::ArgumentError::ResourceNotFound(
                "--json flag is required".to_string(),
            )));
        }
        _ => {}
    }

    let api_client = get_api_client(proxy.clone())?;

    match action {
        EntryAction::Current | EntryAction::Running => {
            RunningTimeEntryCommand::execute(api_client).await
        }
        EntryAction::List {
            number,
            json,
            since,
            until,
        } => ListCommand::execute(api_client, number, json, since, until, None).await,
        EntryAction::Stop => {
            StopCommand::execute(&api_client, StopCommandOrigin::CommandLine).await?;
            Ok(())
        }
        EntryAction::Start {
            interactive,
            billable,
            description,
            project,
            task,
            tags,
            start,
            end,
        } => {
            StartCommand::execute(
                api_client,
                picker,
                description,
                project,
                task,
                tags,
                billable,
                interactive,
                start,
                end,
            )
            .await
        }
        EntryAction::Continue { interactive } => {
            let picker_option = if interactive { Some(picker) } else { None };
            ContinueCommand::execute(api_client, picker_option).await
        }
        EntryAction::Show { id, json } => {
            // id is guaranteed Some due to validation above
            ShowCommand::execute(api_client, id.unwrap(), json).await
        }
        EntryAction::Update {
            id,
            description,
            billable,
            project,
            task,
            tags,
            start,
            end,
        } => {
            EditCommand::execute(
                api_client,
                id,
                description,
                billable,
                project,
                task,
                tags,
                start,
                end,
            )
            .await
        }
        EntryAction::Delete { id } => {
            // id is guaranteed Some due to validation above
            DeleteCommand::execute(api_client, id.unwrap()).await
        }
        EntryAction::BulkEdit { ids, json } => {
            // json is guaranteed Some due to validation above
            BulkEditTimeEntriesCommand::execute(api_client, ids.clone(), json.clone().unwrap())
                .await
        }
    }
}

async fn execute_project_command(
    action: ProjectAction,
    api_client: impl ApiClient,
) -> ResultWithDefaultError<()> {
    match action {
        ProjectAction::List { json } => {
            ListCommand::execute(
                api_client,
                None,
                json,
                None,
                None,
                Some(arguments::Entity::Project { json }),
            )
            .await
        }
        ProjectAction::Create { name, color } => {
            CreateProjectCommand::execute(api_client, name, color).await
        }
        ProjectAction::Rename { old_name, new_name } => {
            RenameProjectCommand::execute(api_client, old_name, new_name).await
        }
        ProjectAction::Delete { name } => DeleteProjectCommand::execute(api_client, name).await,
    }
}

async fn execute_tag_command(
    action: TagAction,
    api_client: impl ApiClient,
) -> ResultWithDefaultError<()> {
    match action {
        TagAction::List { json } => {
            ListCommand::execute(
                api_client,
                None,
                json,
                None,
                None,
                Some(arguments::Entity::Tag { json }),
            )
            .await
        }
        TagAction::Create { name } => CreateTagCommand::execute(api_client, name).await,
        TagAction::Rename { old_name, new_name } => {
            RenameTagCommand::execute(api_client, old_name, new_name).await
        }
        TagAction::Delete { name } => DeleteTagCommand::execute(api_client, name).await,
    }
}

async fn execute_client_command(
    action: ClientAction,
    api_client: impl ApiClient,
) -> ResultWithDefaultError<()> {
    match action {
        ClientAction::List { json } => {
            ListCommand::execute(
                api_client,
                None,
                json,
                None,
                None,
                Some(arguments::Entity::Client { json }),
            )
            .await
        }
        ClientAction::Create { name } => CreateClientCommand::execute(api_client, name).await,
        ClientAction::Rename { old_name, new_name } => {
            RenameClientCommand::execute(api_client, old_name, new_name).await
        }
        ClientAction::Delete { name } => DeleteClientCommand::execute(api_client, name).await,
    }
}

async fn execute_task_command(
    action: TaskAction,
    api_client: impl ApiClient,
) -> ResultWithDefaultError<()> {
    match action {
        TaskAction::List { json } => {
            ListCommand::execute(
                api_client,
                None,
                json,
                None,
                None,
                Some(arguments::Entity::Task { json }),
            )
            .await
        }
        TaskAction::Create {
            project,
            name,
            active,
            estimated_seconds,
            user_id,
        } => {
            CreateTaskCommand::execute(
                api_client,
                project,
                name,
                active,
                estimated_seconds,
                user_id,
            )
            .await
        }
        TaskAction::Update {
            project,
            name,
            new_name,
            active,
            estimated_seconds,
            user_id,
        } => {
            UpdateTaskCommand::execute(
                api_client,
                project,
                name,
                new_name,
                active,
                estimated_seconds,
                user_id,
            )
            .await
        }
        TaskAction::Rename {
            project,
            old_name,
            new_name,
        } => {
            UpdateTaskCommand::execute(
                api_client,
                project,
                old_name,
                Some(new_name),
                None,
                None,
                None,
            )
            .await
        }
        TaskAction::Delete { project, name } => {
            DeleteTaskCommand::execute(api_client, project, name).await
        }
    }
}

async fn execute_workspace_command(
    action: WorkspaceAction,
    api_client: impl ApiClient,
) -> ResultWithDefaultError<()> {
    match action {
        WorkspaceAction::List { json } => {
            ListCommand::execute(
                api_client,
                None,
                json,
                None,
                None,
                Some(arguments::Entity::Workspace { json }),
            )
            .await
        }
        WorkspaceAction::Create {
            organization_id,
            name,
        } => CreateWorkspaceCommand::execute(api_client, organization_id, name).await,
        WorkspaceAction::Rename { old_name, new_name } => {
            RenameWorkspaceCommand::execute(api_client, old_name, new_name).await
        }
    }
}

async fn execute_organization_command(
    action: OrganizationAction,
    api_client: impl ApiClient,
) -> ResultWithDefaultError<()> {
    match action {
        OrganizationAction::List { json } => {
            OrganizationCommand::execute(
                api_client,
                commands::organization::OrganizationAction::List { json },
            )
            .await
        }
        OrganizationAction::Show { id, json } => {
            OrganizationCommand::execute(
                api_client,
                commands::organization::OrganizationAction::Show { id, json },
            )
            .await
        }
    }
}

async fn execute_preferences_command(
    action: PreferencesAction,
    api_client: impl ApiClient,
) -> ResultWithDefaultError<()> {
    match action {
        PreferencesAction::Read => PreferencesCommand::execute(api_client).await,
        PreferencesAction::Update { json } => {
            UpdatePreferencesCommand::execute(api_client, json).await
        }
    }
}

async fn execute_config_command(
    edit: bool,
    delete: bool,
    path: bool,
    cmd: Option<ConfigAction>,
) -> ResultWithDefaultError<()> {
    match cmd {
        Some(ConfigAction::Init) => config::init::ConfigInitCommand::execute(edit).await,
        Some(ConfigAction::Active) => config::active::ConfigActiveCommand::execute().await,
        None => config::manage::ConfigManageCommand::execute(delete, edit, path).await,
    }
}

async fn execute_auth_command(
    api_token: Option<String>,
    api_type: Option<String>,
    api_url: Option<String>,
    proxy: Option<String>,
) -> ResultWithDefaultError<()> {
    let resolved_api_url = match (api_type.as_deref(), api_url) {
        (_, Some(url)) => Some(url),
        (Some("official"), None) => None,
        (Some("opentoggl"), None) => {
            println!("Enter OpenToggl API URL:");
            let url = utilities::read_from_stdin("> ");
            if url.is_empty() {
                eprintln!("URL cannot be empty.");
                return Ok(());
            }
            Some(url)
        }
        (Some(t), None) => {
            eprintln!("Invalid --api-type '{}'. Use 'official' or 'opentoggl'.", t);
            return Ok(());
        }
        (None, None) => {
            println!("Select Toggl service provider:");
            println!("  1) Official Toggl Track (default)");
            println!("  2) OpenToggl (self-hosted)");
            let choice = read_from_stdin_with_constraints(
                "Enter choice (1/2) [1]: ",
                &["1".to_string(), "2".to_string(), "".to_string()],
            );
            match choice.as_str() {
                "" | "1" => None,
                "2" => {
                    println!("Enter OpenToggl API URL:");
                    let url = utilities::read_from_stdin("> ");
                    if url.is_empty() {
                        eprintln!("URL cannot be empty.");
                        return Ok(());
                    }
                    Some(url)
                }
                _ => return Ok(()),
            }
        }
    };

    let api_token = match api_token {
        Some(t) => t,
        None => {
            println!("Enter your Toggl API token:");
            utilities::read_from_stdin("> ")
        }
    };

    if api_token.is_empty() {
        eprintln!("API token cannot be empty.");
        return Ok(());
    }

    let credentials = Credentials {
        api_token,
        api_url: resolved_api_url.clone(),
    };
    let api_client = V9ApiClient::from_credentials(credentials.clone(), proxy)?;
    let storage = get_storage();
    AuthenticationCommand::execute(io::stdout(), api_client, storage, resolved_api_url).await
}

async fn execute_logout_command() -> ResultWithDefaultError<()> {
    let storage = get_storage();
    storage.clear()?;
    println!("Successfully logged out.");
    Ok(())
}

fn get_api_client(proxy: Option<String>) -> ResultWithDefaultError<impl ApiClient> {
    let credentials = CACHED_CREDENTIALS.get_or_try_init(|| {
        let storage = get_storage();
        storage.read()
    })?;
    V9ApiClient::from_credentials(credentials.clone(), proxy)
}
