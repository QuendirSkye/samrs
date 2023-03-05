/*
 * This file is part of samrs
 *
 * Licensed under GPL-3.0
 * See LICENSE for licensing information.
 */

use reqwest;
use serde::{Deserialize, Serialize};

use crate::SAMError;

#[derive(Clone, Deserialize, Serialize)]
pub struct AppEntry {
    pub appid: u32,
}

#[derive(Deserialize, Serialize)]
pub struct AppList {
    pub apps: Vec<AppEntry>,
}

#[derive(Deserialize, Serialize)]
pub struct AppListResponse {
    pub applist: AppList,
}

const APP_LIST_URL: &str = "https://api.steampowered.com/ISteamApps/GetAppList/v2/";

pub async fn fetch_app_list() -> Result<AppList, SAMError> {
    let resp = reqwest::get(APP_LIST_URL).await;

    match resp {
        Err(_) => Err(SAMError::AppListRequestError),
        Ok(res) => match res.json::<AppListResponse>().await {
            Err(err) => Err(SAMError::AppListDeserializationError(err.to_string())),
            Ok(res) => Ok(res.applist),
        },
    }
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
pub async fn filter_app_list_game_w_achievements(
    app_list: &AppList,
    progress: impl FnOnce(usize, usize) + Copy,
) -> Vec<AppEntry> {
    let mut filtered_app_list: Vec<AppEntry> = vec![];

    let total = app_list.apps.len().clone();

    for entry in &app_list.apps {
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
                        tokio::time::sleep(std::time::Duration::from_millis(1000 * 60 * 2)).await;
                        continue;
                    } else if response.status() == 502 {
                        // randomly got bad gateway, wait 500ms
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
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

        progress(total, filtered_app_list.len());
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    filtered_app_list
}
