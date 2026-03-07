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
use std::io::{self, Write};
use structopt::StructOpt;

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
            Command::Stop => {
                StopCommand::execute(&get_default_api_client()?, StopCommandOrigin::CommandLine)
                    .await?;
            }

            Command::Continue { interactive } => {
                let picker = if interactive { Some(picker) } else { None };
                ContinueCommand::execute(get_default_api_client()?, picker).await?
            }

            Command::List {
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

            Command::Current | Command::Running => {
                RunningTimeEntryCommand::execute(get_default_api_client()?).await?
            }

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
                    get_default_api_client()?,
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
                .await?
            }

            Command::Edit { entity } => match entity {
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
                        get_default_api_client()?,
                        id,
                        description,
                        billable,
                        project,
                        task,
                        tags,
                        start,
                        end,
                    )
                    .await?
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
                EditEntity::Preferences { json } => {
                    UpdatePreferencesCommand::execute(get_default_api_client()?, json).await?
                }
            },

            Command::Delete { entity, id } => match entity {
                Some(delete_entity) => match delete_entity {
                    DeleteEntity::Project { name } => {
                        DeleteProjectCommand::execute(get_default_api_client()?, name).await?
                    }
                    DeleteEntity::Tag { name } => {
                        DeleteTagCommand::execute(get_default_api_client()?, name).await?
                    }
                    DeleteEntity::Client { name } => {
                        DeleteClientCommand::execute(get_default_api_client()?, name).await?
                    }
                    DeleteEntity::Task { project, name } => {
                        DeleteTaskCommand::execute(get_default_api_client()?, project, name).await?
                    }
                },
                None => match id {
                    Some(id) => DeleteCommand::execute(get_default_api_client()?, id).await?,
                    None => print_delete_help()?,
                },
            },

            Command::BulkEditTimeEntries { ids, json } => match json {
                Some(json) => {
                    BulkEditTimeEntriesCommand::execute(get_default_api_client()?, ids, json)
                        .await?
                }
                None => print_bulk_edit_time_entries_help()?,
            },

            Command::Create { entity } => match entity {
                CreateEntity::Project { name, color } => {
                    CreateProjectCommand::execute(get_default_api_client()?, name, color).await?
                }
                CreateEntity::Tag { name } => {
                    CreateTagCommand::execute(get_default_api_client()?, name).await?
                }
                CreateEntity::Client { name } => {
                    CreateClientCommand::execute(get_default_api_client()?, name).await?
                }
                CreateEntity::Workspace {
                    organization_id,
                    name,
                } => {
                    CreateWorkspaceCommand::execute(
                        get_default_api_client()?,
                        organization_id,
                        name,
                    )
                    .await?
                }
                CreateEntity::Task {
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
            },

            Command::Rename { entity } => match entity {
                RenameEntity::Project { old_name, new_name } => {
                    RenameProjectCommand::execute(get_default_api_client()?, old_name, new_name)
                        .await?
                }
                RenameEntity::Tag { old_name, new_name } => {
                    RenameTagCommand::execute(get_default_api_client()?, old_name, new_name).await?
                }
                RenameEntity::Client { old_name, new_name } => {
                    RenameClientCommand::execute(get_default_api_client()?, old_name, new_name)
                        .await?
                }
                RenameEntity::Workspace { old_name, new_name } => {
                    RenameWorkspaceCommand::execute(get_default_api_client()?, old_name, new_name)
                        .await?
                }
            },

            Command::Show { id, json } => match id {
                Some(id) => ShowCommand::execute(get_default_api_client()?, id, json).await?,
                None => print_show_help()?,
            },

            Command::Me => MeCommand::execute(get_default_api_client()?).await?,

            Command::Organization { entity } => match entity {
                Some(OrganizationEntity::List { json }) => {
                    OrganizationCommand::execute(
                        get_default_api_client()?,
                        OrganizationAction::List { json },
                    )
                    .await?
                }
                Some(OrganizationEntity::Show { id, json }) => {
                    OrganizationCommand::execute(
                        get_default_api_client()?,
                        OrganizationAction::Show { id, json },
                    )
                    .await?
                }
                None => print_organization_help()?,
            },

            Command::Preferences => PreferencesCommand::execute(get_default_api_client()?).await?,

            Command::Auth { api_token } => match api_token {
                Some(api_token) => {
                    let credentials = Credentials { api_token };
                    let api_client = V9ApiClient::from_credentials(credentials, args.proxy)?;
                    AuthenticationCommand::execute(io::stdout(), api_client, get_storage()).await?
                }
                None => print_auth_help()?,
            },

            Command::Logout => {
                let storage = get_storage();
                storage.clear()?;
                println!("Successfully logged out.");
            }

            Command::Config {
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

fn print_organization_help() -> ResultWithDefaultError<()> {
    print_nested_help(["toggl", "organization", "--help"])
}

fn print_delete_help() -> ResultWithDefaultError<()> {
    print_nested_help(["toggl", "delete", "--help"])
}

fn print_auth_help() -> ResultWithDefaultError<()> {
    print_nested_help(["toggl", "auth", "--help"])
}

fn print_show_help() -> ResultWithDefaultError<()> {
    print_nested_help(["toggl", "show", "--help"])
}

fn print_bulk_edit_time_entries_help() -> ResultWithDefaultError<()> {
    print_nested_help(["toggl", "bulk-edit-time-entries", "--help"])
}

fn print_nested_help(args: [&str; 3]) -> ResultWithDefaultError<()> {
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
