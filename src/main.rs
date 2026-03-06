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

use api::client::ApiClient;
use api::client::V9ApiClient;
use arguments::Command::Auth;
use arguments::Command::Config;
use arguments::Command::Continue;
use arguments::Command::CreateClient;
use arguments::Command::CreateProject;
use arguments::Command::CreateTag;
use arguments::Command::CreateTask;
use arguments::Command::CreateWorkspace;
use arguments::Command::Current;
use arguments::Command::Delete;
use arguments::Command::DeleteClient;
use arguments::Command::DeleteProject;
use arguments::Command::DeleteTag;
use arguments::Command::DeleteTask;
use arguments::Command::Edit;
use arguments::Command::List;
use arguments::Command::Logout;
use arguments::Command::Me;
use arguments::Command::Preferences;
use arguments::Command::RenameClient;
use arguments::Command::RenameProject;
use arguments::Command::RenameTag;
use arguments::Command::RenameWorkspace;
use arguments::Command::Running;
use arguments::Command::Show;
use arguments::Command::Start;
use arguments::Command::Stop;
use arguments::Command::UpdatePreferences;
use arguments::Command::UpdateTask;
use arguments::CommandLineArguments;
use arguments::ConfigSubCommand;
use commands::auth::AuthenticationCommand;
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
use std::io;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> ResultWithDefaultError<()> {
    let parsed_args = CommandLineArguments::from_args();
    match execute_subcommand(parsed_args).await {
        Ok(()) => Ok(()),
        Err(error) => {
            // We are catching the error and pretty printing it instead of letting the
            // program error. Since we are not meant to be used other programs, I think
            // it's fine to always return a 0 error code, but we might wanna revisit this.
            print!("{error}");
            Ok(())
        }
    }
}

async fn execute_subcommand(args: CommandLineArguments) -> ResultWithDefaultError<()> {
    let command = args.cmd;
    let get_default_api_client = || get_api_client(args.proxy.clone());
    let picker = picker::get_picker(args.fzf);
    if let Some(directory) = args.directory {
        if !directory.exists() {
            return Err(Box::new(error::ArgumentError::DirectoryNotFound(directory)));
        }
        if !directory.is_dir() {
            return Err(Box::new(error::ArgumentError::NotADirectory(directory)));
        }
        std::env::set_current_dir(directory).expect("Couldn't set current directory");
    }
    match command {
        None => RunningTimeEntryCommand::execute(get_default_api_client()?).await?,
        Some(subcommand) => match subcommand {
            Stop => {
                StopCommand::execute(&get_default_api_client()?, StopCommandOrigin::CommandLine)
                    .await?;
            }

            Continue { interactive } => {
                let picker = if interactive { Some(picker) } else { None };
                ContinueCommand::execute(get_default_api_client()?, picker).await?
            }

            List {
                number,
                json,
                since,
                until,
                entity,
            } => {
                ListCommand::execute(
                    get_default_api_client()?,
                    number,
                    json,
                    since,
                    until,
                    entity,
                )
                .await?
            }

            Current | Running => {
                RunningTimeEntryCommand::execute(get_default_api_client()?).await?
            }

            Start {
                interactive,
                billable,
                description,
                project,
                tags,
                start,
                end,
            } => {
                StartCommand::execute(
                    get_default_api_client()?,
                    picker,
                    description,
                    project,
                    tags,
                    billable,
                    interactive,
                    start,
                    end,
                )
                .await?
            }

            Edit {
                id,
                description,
                project,
                tags,
                start,
                end,
            } => {
                EditCommand::execute(
                    get_default_api_client()?,
                    id,
                    description,
                    project,
                    tags,
                    start,
                    end,
                )
                .await?
            }

            Delete { id } => DeleteCommand::execute(get_default_api_client()?, id).await?,

            CreateProject { name, color } => {
                CreateProjectCommand::execute(get_default_api_client()?, name, color).await?
            }

            DeleteProject { name } => {
                DeleteProjectCommand::execute(get_default_api_client()?, name).await?
            }

            RenameProject { old_name, new_name } => {
                RenameProjectCommand::execute(get_default_api_client()?, old_name, new_name).await?
            }

            CreateTag { name } => {
                CreateTagCommand::execute(get_default_api_client()?, name).await?
            }

            DeleteTag { name } => {
                DeleteTagCommand::execute(get_default_api_client()?, name).await?
            }

            RenameTag { old_name, new_name } => {
                RenameTagCommand::execute(get_default_api_client()?, old_name, new_name).await?
            }

            CreateClient { name } => {
                CreateClientCommand::execute(get_default_api_client()?, name).await?
            }

            DeleteClient { name } => {
                DeleteClientCommand::execute(get_default_api_client()?, name).await?
            }

            RenameClient { old_name, new_name } => {
                RenameClientCommand::execute(get_default_api_client()?, old_name, new_name).await?
            }

            CreateWorkspace {
                organization_id,
                name,
            } => {
                CreateWorkspaceCommand::execute(get_default_api_client()?, organization_id, name)
                    .await?
            }

            RenameWorkspace { old_name, new_name } => {
                RenameWorkspaceCommand::execute(get_default_api_client()?, old_name, new_name)
                    .await?
            }

            CreateTask {
                project,
                name,
                active,
                estimated_seconds,
                user_id,
            } => {
                CreateTaskCommand::execute(
                    get_default_api_client()?,
                    project,
                    name,
                    active,
                    estimated_seconds,
                    user_id,
                )
                .await?
            }

            UpdateTask {
                project,
                name,
                new_name,
                active,
                estimated_seconds,
                user_id,
            } => {
                UpdateTaskCommand::execute(
                    get_default_api_client()?,
                    project,
                    name,
                    new_name,
                    active,
                    estimated_seconds,
                    user_id,
                )
                .await?
            }

            DeleteTask { project, name } => {
                DeleteTaskCommand::execute(get_default_api_client()?, project, name).await?
            }

            Show { id, json } => ShowCommand::execute(get_default_api_client()?, id, json).await?,

            Me => MeCommand::execute(get_default_api_client()?).await?,

            Preferences => PreferencesCommand::execute(get_default_api_client()?).await?,

            UpdatePreferences { json } => {
                UpdatePreferencesCommand::execute(get_default_api_client()?, json).await?
            }

            Auth { api_token } => {
                let credentials = Credentials { api_token };
                let api_client = V9ApiClient::from_credentials(credentials, args.proxy)?;
                AuthenticationCommand::execute(io::stdout(), api_client, get_storage()).await?
            }

            Logout => {
                let storage = get_storage();
                storage.clear()?;
                println!("Successfully logged out.");
            }

            Config {
                delete,
                cmd,
                edit,
                path,
            } => match cmd {
                Some(config_command) => match config_command {
                    ConfigSubCommand::Init => {
                        config::init::ConfigInitCommand::execute(edit).await?;
                    }
                    ConfigSubCommand::Active => {
                        config::active::ConfigActiveCommand::execute().await?;
                    }
                },
                None => config::manage::ConfigManageCommand::execute(delete, edit, path).await?,
            },
        },
    }

    Ok(())
}

fn get_api_client(proxy: Option<String>) -> ResultWithDefaultError<impl ApiClient> {
    let credentials_storage = get_storage();
    match credentials_storage.read() {
        Ok(credentials) => V9ApiClient::from_credentials(credentials, proxy),
        Err(err) => Err(err),
    }
}
