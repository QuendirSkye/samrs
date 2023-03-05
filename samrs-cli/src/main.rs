/*
 * This file is part of samrs-cli
 * Copyright (C) 2023 Jonni Liljamo <jonni@liljamo.com>
 *
 * Licensed under GPL-3.0
 * See LICENSE for licensing information.
 */

use clap::{Parser, Subcommand};
use indicatif::{HumanDuration, ProgressBar, ProgressStyle};
use poll_promise::Promise;
use samrs::applist::{fetch_app_list, filter_app_list_game_w_achievements, AppList};
use tokio::{
    fs::File,
    io::{self, AsyncReadExt, AsyncWriteExt},
    runtime,
    time::{Duration, Instant},
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
    DownloadFull {
        #[arg(short)]
        #[arg(default_value = "./app_list_all.json")]
        output_path: String,
    },
    Filter {
        #[arg(short)]
        #[arg(default_value = "./app_list_all.json")]
        input_path: String,
        #[arg(short)]
        #[arg(default_value = "./app_list_game_w_achievements.json")]
        output_path: String,
    },
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
            AppListCmds::DownloadFull { output_path } => {
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
                                            match save_to_file(&output_path, json.as_bytes()).await
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
            AppListCmds::Filter {
                input_path,
                output_path,
            } => {
                //let pb = ProgressBar::new(0);
                //pb.set_style(ProgressStyle::default_bar()
                //    .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})").unwrap()
                //    .progress_chars("#>-"));
                //pb.set_message("Filtering app list...");

                let started = Instant::now();
                let future = async {
                    let mut applist_file = match File::open(input_path).await {
                        Err(_) => panic!("failed to open input file"),
                        Ok(applist) => applist,
                    };

                    let mut applist_contents = vec![];
                    match applist_file.read_to_end(&mut applist_contents).await {
                        Err(_) => panic!("failed to read input file"),
                        Ok(_) => {}
                    }

                    let applist = match serde_json::from_slice::<AppList>(&applist_contents) {
                        Err(_) => panic!("failed to deserialize input file"),
                        Ok(res) => res,
                    };

                    let promise = Promise::spawn_async(async move {
                        filter_app_list_game_w_achievements(applist, move |total, done, status| {
                            println!("Progress {}/{}, {}", done, total, status);
                        })
                        .await
                    });
                    loop {
                        if let Some(filtered_applist) = promise.ready() {
                            //pb.finish_with_message("App list filtered!");

                            let json = serde_json::to_string(filtered_applist);
                            match json {
                                Err(_) => println!("failed to serialize filtered_applist and save"),
                                Ok(json) => {
                                    match save_to_file(&output_path, json.as_bytes()).await {
                                        Err(_) => panic!("failed to save filtered_applist to file"),
                                        Ok(_) => return,
                                    }
                                }
                            }

                            return;
                        } else {
                            //pb.set_length(pbtotal);
                            //pb.set_position(pbdone);
                            tokio::time::sleep(Duration::from_millis(5)).await;
                        }
                    }
                };
                rt.block_on(future);
                println!("took: {}", HumanDuration(started.elapsed()));
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
