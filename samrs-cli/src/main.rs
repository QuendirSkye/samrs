/*
 * This file is part of samrs-cli
 * Copyright (C) 2023 Jonni Liljamo <jonni@liljamo.com>
 *
 * Licensed under GPL-3.0
 * See LICENSE for licensing information.
 */

use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use poll_promise::Promise;
use samrs::applist::{fetch_app_list, filter_app_list_game_w_achievements};
use tokio::{
    fs::File,
    io::{self, AsyncWriteExt},
    runtime,
    time::Duration,
};

#[derive(Parser)]
#[command(name = "samrs-cli")]
#[command(version, author)] // read from Cargo.toml
#[command(about = "Steam Achievement Manager CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(subcommand)]
    AppList(AppListCmds),
}

#[derive(Subcommand)]
enum AppListCmds {
    DownloadFull,
    Filter,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let rt = runtime::Builder::new_current_thread()
        .enable_time()
        .enable_io()
        .build()
        .expect("failed to create runtime");

    match cli.command {
        Commands::AppList(cmds) => match cmds {
            AppListCmds::DownloadFull => {
                let pb = ProgressBar::new(0);
                pb.set_style(
                    ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
                        .unwrap()
                        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "),
                );
                pb.set_message("Fetching app list...");

                let future = async {
                    let promise = Promise::spawn_async(async move { fetch_app_list().await });
                    loop {
                        if let Some(applist) = promise.ready() {
                            match applist {
                                Err(err) => {
                                    match err {
                                        samrs::SAMError::AppListDeserializationError(ierr) => {
                                            pb.finish_with_message(format!(
                                                "Fetching app list failed! '{}'",
                                                ierr
                                            ));
                                        }
                                        _ => {
                                            pb.finish_with_message(format!(
                                                "Fetching app list failed! '{}'",
                                                err
                                            ));
                                        }
                                    }

                                    return;
                                }
                                Ok(applist) => {
                                    pb.finish_with_message("Fetched app list!");

                                    let json = serde_json::to_string(applist);
                                    match json {
                                        Err(_) => println!("failed to serialize applist and save"),
                                        Ok(json) => {
                                            match save_to_file(
                                                "./app_list_all.json",
                                                json.as_bytes(),
                                            )
                                            .await
                                            {
                                                Err(_) => panic!("failed to save applist to file"),
                                                Ok(_) => return,
                                            }
                                        }
                                    }

                                    return;
                                }
                            }
                        } else {
                            pb.tick();
                            tokio::time::sleep(Duration::from_millis(5)).await;
                        }
                    }
                };
                rt.block_on(future);
            }
            AppListCmds::Filter => {
                /*
                let pb = ProgressBar::new(0);
                pb.set_style(ProgressStyle::default_bar()
                    .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})").unwrap()
                    .progress_chars("#>-"));
                pb.set_message("Fetching app list...");

                // TODO: take from input or default to this, yeh?
                let app_list =
                    filter_app_list_game_w_achievements("./app_list_all.json", |total, done| {
                        pb.set_length(total);
                        pb.set_position(done);
                    })
                    .await;
                */
            }
        },
    }

    Ok(())
}

async fn save_to_file(path: &str, content: &[u8]) -> io::Result<()> {
    let mut file = File::create(path).await?;

    file.write_all(content).await?;

    Ok(())
}
