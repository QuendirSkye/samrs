/*
 * This file is part of samrs
 *
 * Licensed under GPL-3.0
 * See LICENSE for licensing information.
 */

use std::cmp::min;

use async_std::{fs::File, io::WriteExt, stream::StreamExt};
use reqwest;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct AppEntry {
    pub appid: u32,
}

#[derive(Deserialize)]
pub struct AppList {
    pub applist: Vec<AppEntry>,
}

const APP_LIST_URL: &str = "https://api.steampowered.com/ISteamApps/GetAppList/v2/";

pub async fn fetch_app_list(path: &str, progress: impl FnOnce(u64, u64) + Copy) -> Result<(), ()> {
    // TODO: lots of errors to handle...

    let resp = reqwest::Client::new()
        .get(APP_LIST_URL)
        .send()
        .await
        .unwrap();

    let total_size = resp.content_length().unwrap_or(0);

    let mut file = File::create(path).await.or(Err(()))?;
    let mut downloaded: u64 = 0;
    let mut stream = resp.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.or(Err(()))?;
        file.write_all(&chunk).await.or(Err(()))?;
        downloaded = min(downloaded + (chunk.len() as u64), total_size);
        progress(total_size, downloaded);
    }

    Ok(())

    /*
    let app_list = reqwest::get(APP_LIST_URL).await;

    match app_list {
        Err(_) => Err(()),
        Ok(response) => match response.json::<AppList>().await {
            Err(_) => Err(()),
            Ok(res) => Ok(res),
        },
    }
    */
}

#[derive(Deserialize)]
pub struct AchievementInfo {
    pub total: u16,
}

#[derive(Deserialize)]
pub struct AppDetails {
    pub r#type: String,
    pub achievements: Option<AchievementInfo>,
}

const APP_DETAILS_URL: &str =
    "https://store.steampowered.com/api/appdetails/?filters=basic,achievements&appids=";

/// Returns only games with achievements
pub async fn filter_app_list_game_w_achievements(app_list: &AppList) -> Vec<AppEntry> {
    let mut filtered_app_list: Vec<AppEntry> = vec![];

    for entry in &app_list.applist {
        let mut ok = false;
        let mut app_details: Option<AppDetails> = None;

        // TODO: an error counter, and simply skip the app if over N number of errors?

        while !ok {
            let app_details_resp =
                reqwest::get(format!("{}{}", APP_DETAILS_URL, entry.appid)).await;

            match app_details_resp {
                Err(_) => {}
                Ok(response) => {
                    if response.status() == 429 {
                        // too many requests, wait 2 minutes
                        async_std::task::sleep(std::time::Duration::from_millis(1000 * 60 * 2))
                            .await;
                        continue;
                    } else if response.status() == 502 {
                        // randomly got bad gateway, wait 500ms
                        async_std::task::sleep(std::time::Duration::from_millis(500)).await;
                        continue;
                    }

                    match response.json::<AppDetails>().await {
                        Err(_) => {
                            // apparently some urls return absolutely nothing,
                            // so most likely that's what happened.
                            // e.g. https://store.steampowered.com/api/appdetails/?filters=basic,achievements&appids=1444140
                            ok = true;
                        }
                        Ok(details) => app_details = Some(details),
                    }
                }
            }
        }

        if let Some(app_details) = app_details {
            if app_details.r#type == String::from("game") && app_details.achievements.is_some() {
                filtered_app_list.push(entry.clone());
            }
        }

        async_std::task::sleep(std::time::Duration::from_millis(100)).await;
    }

    filtered_app_list
}
