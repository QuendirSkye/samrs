/*
 * This file is part of samrs-cli
 * Copyright (C) 2023 Jonni Liljamo <jonni@liljamo.com>
 *
 * Licensed under GPL-3.0
 * See LICENSE for licensing information.
 */

use clap::{command, Parser, Subcommand};
use indicatif::HumanDuration;
use samrs::{SAMGame, SAM};
use tokio::{
    fs::File,
    io::{self, AsyncReadExt, AsyncWriteExt},
    runtime,
};

mod applist;

#[derive(Parser)]
#[command(name = "samrs-cli")]
#[command(version, author)] // read from Cargo.toml
#[command(about = "Steam Achievement Manager CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
#[command(rename_all = "lower")]
enum Commands {
    #[command(subcommand)]
    #[command(about = "")]
    AppList(AppListCmds),
    #[command(about = "List all games on current account")]
    ListGames {
        #[arg(short = 'i', long = "input")]
        #[arg(default_value = "./app_list_game_w_achievements.json")]
        input_path: String,
    },
    #[command(about = "Manage apps on current account")]
    #[command(arg_required_else_help(true))]
    App {
        #[command(subcommand)]
        command: AppCmds,

        #[arg(short = 'i', long = "appid")]
        #[arg(help = "App ID of the app to manage")]
        appid: Option<usize>,
    },
}

#[derive(Subcommand)]
#[command(rename_all = "kebab-case")]
enum AppCmds {
    #[command(about = "List all achievements for the given App ID")]
    List,
    #[command(about = "Set the state of a specific achievement for the given App ID")]
    Set,
    #[command(about = "Set the state of all achievements for the given App ID")]
    SetAll,
}

#[derive(Subcommand)]
#[command(rename_all = "lower")]
enum AppListCmds {
    #[command(about = "Download all app ids")]
    DlAll {
        #[arg(short = 'o', long = "output")]
        #[arg(help = "Where to store the output")]
        #[arg(default_value = "./app_list_all.json")]
        output_path: String,
    },
}

fn main() -> io::Result<()> {
    let rt = runtime::Builder::new_current_thread()
        .enable_time()
        .enable_io()
        .build()
        .expect("failed to create runtime");

    let cli = Cli::parse();

    match cli.command {
        Commands::AppList(cmd) => match cmd {
            AppListCmds::DlAll { output_path } => {
                applist::download_all(&rt, &output_path);
            }
        },
        Commands::ListGames { input_path } => {
            let started = tokio::time::Instant::now();
            let future = async {
                let mut applist_file = match tokio::fs::File::open(input_path).await {
                    Err(_) => panic!("failed to open input file"),
                    Ok(applist) => applist,
                };

                let mut applist_contents = vec![];
                match applist_file.read_to_end(&mut applist_contents).await {
                    Err(_) => panic!("failed to read input file"),
                    Ok(_) => {}
                }

                let applist = match serde_json::from_slice::<Vec<SAMGame>>(&applist_contents) {
                    Err(_) => panic!("failed to deserialize input file"),
                    Ok(res) => res,
                };

                let sam_res = SAM::init();
                match sam_res {
                    Err(_) => panic!("sam failed to init"),
                    Ok(mut sam) => {
                        _ = sam.populate_user_games(applist);
                        let owned_games = sam.get_user_games();

                        for game in owned_games {
                            println!("{}\t : {}", game.appid, game.name);
                        }
                    }
                }
            };
            rt.block_on(future);
            println!("took: {}", HumanDuration(started.elapsed()));
        }
        Commands::App { command, appid } => {
            let appid = match appid {
                None => panic!("appid must be provided"),
                Some(appid) => appid,
            };

            match command {
                AppCmds::List => {
                    //
                }
                AppCmds::Set => {
                    //
                }
                AppCmds::SetAll => {
                    //
                }
            }
        }
    }

    Ok(())
}

pub async fn save_to_file(path: &str, content: &[u8]) -> io::Result<()> {
    let mut file = File::create(path).await?;

    file.write_all(content).await?;

    Ok(())
}
