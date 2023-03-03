/*
 * This file is part of samrs-cli
 * Copyright (C) 2023 Jonni Liljamo <jonni@liljamo.com>
 *
 * Licensed under GPL-3.0
 * See LICENSE for licensing information.
 */

use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use samrs::applist::fetch_app_list;

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

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::AppList(cmds) => match cmds {
            AppListCmds::DownloadFull => {
                let pb = ProgressBar::new(192837192);
                pb.set_style(ProgressStyle::default_bar()
                    .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})").unwrap()
                    .progress_chars("#>-"));
                pb.set_message("Fetching app list...");

                // TODO: take from input or default to this, yeh?
                let app_list = fetch_app_list("./app_list_all.json", |total, downloaded| {
                    pb.set_length(total);
                    pb.set_position(downloaded);
                })
                .await;
            }
            AppListCmds::Filter => {
                //fetch_app_list(|cur_idx, total| {
                //progress_bar("filtered id/total (total is gotten from the len of the thingy vec)")
                //}),
            }
        },
    }
}
