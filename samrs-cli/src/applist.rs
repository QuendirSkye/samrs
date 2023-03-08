/*
 * This file is part of samrs-cli
 * Copyright (C) 2023 Jonni Liljamo <jonni@liljamo.com>
 *
 * Licensed under GPL-3.0
 * See LICENSE for licensing information.
 */

use indicatif::{ProgressBar, ProgressStyle};
use poll_promise::Promise;
use samrs::applist::fetch_applist;
use tokio::runtime::Runtime;

use crate::save_to_file;

pub fn download_all(rt: &Runtime, output_path: &str) {
    let pb = ProgressBar::new(0);
    pb.set_style(
        ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
            .unwrap()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "),
    );
    pb.set_message("Fetching applist...");

    let future = async {
        let promise = Promise::spawn_async(async move { fetch_applist().await });
        loop {
            if let Some(applist) = promise.ready() {
                match applist {
                    Err(err) => {
                        match err {
                            samrs::SAMError::AppListDeserializationError(ierr) => {
                                pb.finish_with_message(format!(
                                    "Fetching applist failed! '{}'",
                                    ierr
                                ));
                            }
                            _ => {
                                pb.finish_with_message(format!(
                                    "Fetching applist failed! '{}'",
                                    err
                                ));
                            }
                        }

                        return;
                    }
                    Ok(applist) => {
                        pb.finish_with_message("Fetched applist!");

                        let json = serde_json::to_string(applist);
                        match json {
                            Err(_) => println!("failed to serialize applist and save"),
                            Ok(json) => match save_to_file(&output_path, json.as_bytes()).await {
                                Err(_) => panic!("failed to save applist to file"),
                                Ok(_) => return,
                            },
                        }

                        return;
                    }
                }
            } else {
                pb.tick();
                tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
            }
        }
    };
    rt.block_on(future);
}

/*
// related to NOTE in samrs/src/applist.rs

pub fn filter(rt: &Runtime, output_path: &str, input_path: &str) {
    //let pb = ProgressBar::new(0);
    //pb.set_style(ProgressStyle::default_bar()
    //    .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})").unwrap()
    //    .progress_chars("#>-"));
    //pb.set_message("Filtering appids...");

    let started = tokio::time::Instant::now();
    let future = async {
        let mut appids_file = match tokio::fs::File::open(input_path).await {
            Err(_) => panic!("failed to open input file"),
            Ok(appids) => appids,
        };

        let mut appids_contents = vec![];
        match appids_file.read_to_end(&mut appids_contents).await {
            Err(_) => panic!("failed to read input file"),
            Ok(_) => {}
        }

        let appids = match serde_json::from_slice::<AppIds>(&appids_contents) {
            Err(_) => panic!("failed to deserialize input file"),
            Ok(res) => res,
        };

        let promise = Promise::spawn_async(async move {
            filter_app_ids_game_w_achievements(appids, move |total, done, status| {
                println!("Progress {}/{}, {}", done, total, status);
            })
            .await
        });
        loop {
            if let Some(filtered_appids) = promise.ready() {
                //pb.finish_with_message("Appids filtered!");

                let json = serde_json::to_string(filtered_appids);
                match json {
                    Err(_) => println!("failed to serialize filtered_appids and save"),
                    Ok(json) => match save_to_file(&output_path, json.as_bytes()).await {
                        Err(_) => panic!("failed to save filtered_appids to file"),
                        Ok(_) => return,
                    },
                }

                return;
            } else {
                //pb.set_length(pbtotal);
                //pb.set_position(pbdone);
                tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
            }
        }
    };
    rt.block_on(future);
    println!("took: {}", HumanDuration(started.elapsed()));
}
*/
