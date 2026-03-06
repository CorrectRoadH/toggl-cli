use crate::api::client::ApiClient;
use crate::models::ResultWithDefaultError;
use colored::Colorize;
use std::io::{self, BufWriter, Write};

pub struct ShowCommand;

impl ShowCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        id: i64,
        json: bool,
    ) -> ResultWithDefaultError<()> {
        match api_client.get_time_entry(id).await {
            Err(error) => println!(
                "{}\n{}",
                format!("Couldn't fetch time entry with ID {id}").red(),
                error
            ),
            Ok(entry) => {
                let stdout = io::stdout();
                let mut handle = BufWriter::new(stdout);
                if json {
                    let json_string = serde_json::to_string_pretty(&entry)
                        .expect("failed to serialize time entry to JSON");
                    writeln!(handle, "{json_string}").expect("failed to print");
                } else {
                    writeln!(handle, "{}", "Time Entry Details".bold().underline())
                        .expect("failed to print");
                    writeln!(handle, "  {} {}", "ID:".bold(), entry.id).expect("failed to print");
                    writeln!(
                        handle,
                        "  {} {}",
                        "Description:".bold(),
                        entry.get_description()
                    )
                    .expect("failed to print");
                    writeln!(handle, "  {} {}", "Start:".bold(), entry.start)
                        .expect("failed to print");
                    match entry.stop {
                        Some(stop) => {
                            writeln!(handle, "  {} {}", "Stop:".bold(), stop)
                                .expect("failed to print");
                        }
                        None => {
                            writeln!(
                                handle,
                                "  {} {}",
                                "Status:".bold(),
                                "Running".green().bold()
                            )
                            .expect("failed to print");
                        }
                    }
                    writeln!(
                        handle,
                        "  {} {}",
                        "Duration:".bold(),
                        entry.get_duration_hmmss()
                    )
                    .expect("failed to print");
                    writeln!(
                        handle,
                        "  {} {}",
                        "Billable:".bold(),
                        if entry.billable { "Yes" } else { "No" }
                    )
                    .expect("failed to print");
                    writeln!(
                        handle,
                        "  {} {}",
                        "Workspace ID:".bold(),
                        entry.workspace_id
                    )
                    .expect("failed to print");
                    if !entry.tags.is_empty() {
                        writeln!(handle, "  {} {}", "Tags:".bold(), entry.tags.join(", "))
                            .expect("failed to print");
                    }
                }
            }
        }
        Ok(())
    }
}
