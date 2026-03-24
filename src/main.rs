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
use arguments::CommandLineArguments;
use arguments::ConfigSubCommand;
use arguments::{
    Command, CreateEntity, DeleteEntity, EditEntity, OrganizationEntity, RenameEntity,
};
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
use commands::organization::{OrganizationAction, OrganizationCommand};
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
use std::io::{self, Write};
use structopt::StructOpt;

static CACHED_CREDENTIALS: OnceCell<Credentials> = OnceCell::new();

#[tokio::main]
async fn main() -> ResultWithDefaultError<()> {
    let parsed_args = CommandLineArguments::from_args();
    match execute_subcommand(parsed_args).await {
        Ok(()) => Ok(()),
        Err(error) => {
            eprint!("{error}");
            std::process::exit(1);
        }
    }
}

async fn execute_subcommand(args: CommandLineArguments) -> ResultWithDefaultError<()> {
    setup_working_directory(args.directory)?;

    let cmd = args.cmd;
    if let Some(Command::Auth {
        api_token,
        api_type,
    }) = cmd
    {
        return execute_auth_command(api_token, api_type, args.proxy).await;
    }

    let api_client = get_api_client(args.proxy.clone())?;
    let picker = picker::get_picker(args.fzf);

    match cmd {
        None => RunningTimeEntryCommand::execute(api_client).await,
        Some(subcommand) => execute_command(subcommand, api_client, picker, args.proxy).await,
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

async fn execute_command(
    command: Command,
    api_client: impl ApiClient,
    picker: Box<dyn picker::ItemPicker>,
    proxy: Option<String>,
) -> ResultWithDefaultError<()> {
    match command {
        Command::Stop => {
            StopCommand::execute(&api_client, StopCommandOrigin::CommandLine).await?;
            Ok(())
        }

        Command::Continue { interactive } => {
            let picker_option = if interactive { Some(picker) } else { None };
            ContinueCommand::execute(api_client, picker_option).await
        }

        Command::List {
            number,
            json,
            since,
            until,
            entity,
        } => ListCommand::execute(api_client, number, json, since, until, entity).await,

        Command::Current | Command::Running => RunningTimeEntryCommand::execute(api_client).await,

        Command::Start {
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

        Command::Edit { entity } => execute_edit_command(entity, api_client).await,

        Command::Delete { entity, id } => execute_delete_command(entity, id, api_client).await,

        Command::BulkEditTimeEntries { ids, json } => {
            execute_bulk_edit_command(ids, json, api_client).await
        }

        Command::Create { entity } => execute_create_command(entity, api_client).await,

        Command::Rename { entity } => execute_rename_command(entity, api_client).await,

        Command::Show { id, json } => execute_show_command(id, json, api_client).await,

        Command::Me => MeCommand::execute(api_client).await,

        Command::Organization { entity } => execute_organization_command(entity, api_client).await,

        Command::Preferences => PreferencesCommand::execute(api_client).await,

        Command::Auth {
            api_token,
            api_type,
        } => execute_auth_command(api_token, api_type, proxy).await,

        Command::Logout => execute_logout_command().await,

        Command::Config {
            delete,
            cmd,
            edit,
            path,
        } => execute_config_command(cmd, delete, edit, path).await,
    }
}

async fn execute_edit_command(
    entity: EditEntity,
    api_client: impl ApiClient,
) -> ResultWithDefaultError<()> {
    match entity {
        EditEntity::TimeEntry {
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
        EditEntity::Task {
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
        EditEntity::Preferences { json } => {
            UpdatePreferencesCommand::execute(api_client, json).await
        }
    }
}

async fn execute_delete_command(
    entity: Option<DeleteEntity>,
    id: Option<i64>,
    api_client: impl ApiClient,
) -> ResultWithDefaultError<()> {
    match entity {
        Some(delete_entity) => match delete_entity {
            DeleteEntity::Project { name } => DeleteProjectCommand::execute(api_client, name).await,
            DeleteEntity::Tag { name } => DeleteTagCommand::execute(api_client, name).await,
            DeleteEntity::Client { name } => DeleteClientCommand::execute(api_client, name).await,
            DeleteEntity::Task { project, name } => {
                DeleteTaskCommand::execute(api_client, project, name).await
            }
        },
        None => match id {
            Some(id) => DeleteCommand::execute(api_client, id).await,
            None => print_help_message("delete"),
        },
    }
}

async fn execute_bulk_edit_command(
    ids: Vec<i64>,
    json: Option<String>,
    api_client: impl ApiClient,
) -> ResultWithDefaultError<()> {
    match json {
        Some(json) => BulkEditTimeEntriesCommand::execute(api_client, ids, json).await,
        None => print_help_message("bulk-edit-time-entries"),
    }
}

async fn execute_create_command(
    entity: CreateEntity,
    api_client: impl ApiClient,
) -> ResultWithDefaultError<()> {
    match entity {
        CreateEntity::Project { name, color } => {
            CreateProjectCommand::execute(api_client, name, color).await
        }
        CreateEntity::Tag { name } => CreateTagCommand::execute(api_client, name).await,
        CreateEntity::Client { name } => CreateClientCommand::execute(api_client, name).await,
        CreateEntity::Workspace {
            organization_id,
            name,
        } => CreateWorkspaceCommand::execute(api_client, organization_id, name).await,
        CreateEntity::Task {
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
    }
}

async fn execute_rename_command(
    entity: RenameEntity,
    api_client: impl ApiClient,
) -> ResultWithDefaultError<()> {
    match entity {
        RenameEntity::Project { old_name, new_name } => {
            RenameProjectCommand::execute(api_client, old_name, new_name).await
        }
        RenameEntity::Tag { old_name, new_name } => {
            RenameTagCommand::execute(api_client, old_name, new_name).await
        }
        RenameEntity::Client { old_name, new_name } => {
            RenameClientCommand::execute(api_client, old_name, new_name).await
        }
        RenameEntity::Workspace { old_name, new_name } => {
            RenameWorkspaceCommand::execute(api_client, old_name, new_name).await
        }
    }
}

async fn execute_show_command(
    id: Option<i64>,
    json: bool,
    api_client: impl ApiClient,
) -> ResultWithDefaultError<()> {
    match id {
        Some(id) => ShowCommand::execute(api_client, id, json).await,
        None => print_help_message("show"),
    }
}

async fn execute_organization_command(
    entity: Option<OrganizationEntity>,
    api_client: impl ApiClient,
) -> ResultWithDefaultError<()> {
    match entity {
        Some(OrganizationEntity::List { json }) => {
            OrganizationCommand::execute(api_client, OrganizationAction::List { json }).await
        }
        Some(OrganizationEntity::Show { id, json }) => {
            OrganizationCommand::execute(api_client, OrganizationAction::Show { id, json }).await
        }
        None => print_help_message("organization"),
    }
}

async fn execute_auth_command(
    api_token: Option<String>,
    api_type: Option<String>,
    proxy: Option<String>,
) -> ResultWithDefaultError<()> {
    let resolved_api_url = match api_type.as_deref() {
        Some("official") => None,
        Some("opentoggl") => {
            println!("Enter OpenToggl API URL:");
            let url = utilities::read_from_stdin("> ");
            if url.is_empty() {
                eprintln!("URL cannot be empty.");
                return Ok(());
            }
            Some(url)
        }
        Some(t) => {
            eprintln!("Invalid --type '{}'. Use 'official' or 'opentoggl'.", t);
            return Ok(());
        }
        None => {
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

async fn execute_config_command(
    cmd: Option<ConfigSubCommand>,
    delete: bool,
    edit: bool,
    path: bool,
) -> ResultWithDefaultError<()> {
    match cmd {
        Some(config_command) => match config_command {
            ConfigSubCommand::Init => config::init::ConfigInitCommand::execute(edit).await,
            ConfigSubCommand::Active => config::active::ConfigActiveCommand::execute().await,
        },
        None => config::manage::ConfigManageCommand::execute(delete, edit, path).await,
    }
}

fn get_api_client(proxy: Option<String>) -> ResultWithDefaultError<impl ApiClient> {
    let credentials = CACHED_CREDENTIALS.get_or_try_init(|| {
        let storage = get_storage();
        storage.read()
    })?;
    V9ApiClient::from_credentials(credentials.clone(), proxy)
}

fn print_help_message(command: &str) -> ResultWithDefaultError<()> {
    let args = ["toggl", command, "--help"];
    match CommandLineArguments::from_iter_safe(args) {
        Ok(_) => Ok(()),
        Err(error) => {
            error
                .write_to(&mut io::stdout())
                .map_err(|err| -> Box<dyn std::error::Error + Send> { Box::new(err) })?;
            io::stdout()
                .flush()
                .map_err(|err| -> Box<dyn std::error::Error + Send> { Box::new(err) })?;
            Ok(())
        }
    }
}
