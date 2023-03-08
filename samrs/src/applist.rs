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
    pub appid: usize,
    pub name: String,
}

#[derive(Deserialize, Serialize)]
struct AppList {
    pub apps: Vec<AppEntry>,
}

#[derive(Deserialize, Serialize)]
struct AppListResponse {
    pub applist: AppList,
}

const APP_LIST_URL: &str = "https://api.steampowered.com/ISteamApps/GetAppList/v2/";

pub async fn fetch_applist() -> Result<Vec<AppEntry>, SAMError> {
    let resp = reqwest::get(APP_LIST_URL).await;

    match resp {
        Err(_) => Err(SAMError::AppListRequestError),
        Ok(res) => match res.json::<AppListResponse>().await {
            Err(err) => Err(SAMError::AppListDeserializationError(err.to_string())),
            Ok(res) => {
                let mut apps = vec![];
                for entry in res.applist.apps {
                    apps.push(AppEntry {
                        appid: entry.appid as usize,
                        name: entry.name,
                    });
                }
                Ok(apps)
            }
        },
    }
}

/*
// TODO/TOFIX/NOTE/WHATEVER
// Supposed to be in use with the following in the README.md:
// ## Notices:
// The `samrs/src/applist.rs` implementation is based on [PaulCombal/SteamAppsList](https://github.com/PaulCombal/SteamAppsList).
//
// But this is currently in fact, not in use.
// I don't really have an idea on what do for filtering down the apps, since this actually takes forever.
//
// It could be written in a way that it would be possible to run it as a Github Action,
// which runs automatically once a month, or manually on request.
// That could then upload the filtered down list somewhere, either Git LFS or somewhere else.
//
// Not a major issue, since it only takes around 3 seconds (on my machine, at least) to
// compare the full list of almost a 160k ids to my library.
//

#[derive(Deserialize)]
pub struct AchievementInfo {
    pub total: u16,
}

#[derive(Deserialize)]
pub struct AppDetails {
    pub r#type: String,
    pub achievements: bool,
}

#[derive(Deserialize)]
pub struct AppDetailsResponse {
    pub details: AppDetails,
}

const APP_DETAILS_URL: &str =
    "https://store.steampowered.com/api/appdetails/?filters=basic,achievements&appids=";

/// Returns only games with achievements
pub async fn filter_applist_game_w_achievements(
    applist: Vec<AppEntry>,
    progress: impl FnOnce(u64, u64, &str) + Copy,
) -> Vec<AppEntry> {
    let mut filtered_applist: Vec<AppEntry> = vec![];

    let mut status = "";
    let total = applist.len().clone();
    let mut done: u64 = 0;

    for entry in &applist {
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
                        status = "429";
                        // too many requests, wait 2 minutes
                        tokio::time::sleep(std::time::Duration::from_millis(1000 * 60 * 2)).await;
                        continue;
                    } else if response.status() == 502 {
                        status = "502";
                        // randomly got bad gateway, wait 500ms
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        continue;
                    }

                    let bytes_res = response.bytes().await;
                    match bytes_res {
                        Err(_) => {
                            status = "not sure";
                            ok = true;
                        }
                        Ok(bytes) => {
                            let json_res: Result<serde_json::Value, serde_json::Error> =
                                serde_json::from_slice(&bytes);
                            match json_res {
                                Err(_) => {
                                    // apparently some urls return absolutely nothing,
                                    // so most likely that's what happened.
                                    // e.g. https://store.steampowered.com/api/appdetails/?filters=basic,achievements&appids=1444140
                                    status = "nothing";
                                    ok = true;
                                }
                                Ok(json) => {
                                    app_details = Some(AppDetails {
                                        r#type: json["type"].to_string(),
                                        achievements: json["ahievements"].is_object(),
                                    });
                                    ok = true;
                                }
                            }
                        }
                    };
                }
            }
        }

        if let Some(app_details) = app_details {
            if app_details.r#type == String::from("game") && app_details.achievements {
                filtered_applist.push(entry.clone());
            }
        }

        done += 1;
        progress(total as u64, done, status);
        status = "";
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    filtered_applist
}
*/
