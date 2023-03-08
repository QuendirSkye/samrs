/*
 * This file is part of samrs
 * Copyright (C) 2023 Jonni Liljamo <jonni@liljamo.com>
 *
 * Licensed under GPL-3.0
 * See LICENSE for licensing information.
 */

use serde::{Deserialize, Serialize};
use steamworks::sys::{
    AppId_t, HSteamPipe, HSteamUser, ISteamApps, SteamAPI_ISteamApps_BIsSubscribedApp,
    SteamAPI_ISteamClient_ConnectToGlobalUser, SteamAPI_ISteamClient_CreateSteamPipe,
    SteamAPI_ISteamClient_GetISteamApps,
};
use thiserror::Error;

pub mod applist;

pub mod utils;

#[allow(dead_code)]
pub struct SAM {
    steamclient_lib: libloading::Library,
    i_steam_client: *mut std::os::raw::c_void,
    h_steam_pipe: HSteamPipe,
    h_steam_user: HSteamUser,
    i_steam_apps: *mut ISteamApps,

    user_games: Vec<SAMGame>,
}

#[derive(Deserialize, Serialize)]
pub struct SAMGame {
    pub appid: usize,
    pub name: String,
}

const STEAMCLIENT_LIB_LINUX: &str = "/linux64/steamclient.so";

const STEAMCLIENT_INTERFACE_VERSION: &str = "SteamClient020";
const STEAMAPPS_INTERFACE_VERSION: &str = "STEAMAPPS_INTERFACE_VERSION008";

impl SAM {
    pub fn init() -> Result<Self, SAMError> {
        let sc_i_v = std::ffi::CString::new(STEAMCLIENT_INTERFACE_VERSION).unwrap();
        let sa_i_v = std::ffi::CString::new(STEAMAPPS_INTERFACE_VERSION).unwrap();

        unsafe {
            let steamclient_lib = match libloading::Library::new(format!(
                "{}{}",
                utils::find_steam_installation_path().unwrap(),
                STEAMCLIENT_LIB_LINUX
            )) {
                Err(err) => return Err(SAMError::ErrorLoadingSteamClientLib(err.to_string())),
                Ok(lib) => lib,
            };

            let fn_create_interface: libloading::Symbol<
                unsafe extern "C" fn(
                    ver: *const std::os::raw::c_char,
                    *const std::os::raw::c_void,
                ) -> *mut std::os::raw::c_void,
            > = match steamclient_lib.get(b"CreateInterface") {
                Err(err) => return Err(SAMError::ErrorGettingCreateInterface(err.to_string())),
                Ok(f) => f,
            };

            let i_steam_client =
                fn_create_interface(sc_i_v.as_ptr(), std::ptr::null::<std::os::raw::c_void>());
            let h_steam_pipe = SteamAPI_ISteamClient_CreateSteamPipe(i_steam_client.cast());
            let h_steam_user =
                SteamAPI_ISteamClient_ConnectToGlobalUser(i_steam_client.cast(), h_steam_pipe);
            let i_steam_apps = SteamAPI_ISteamClient_GetISteamApps(
                i_steam_client.cast(),
                h_steam_user,
                h_steam_pipe,
                sa_i_v.as_ptr(),
            );

            Ok(Self {
                steamclient_lib,
                i_steam_client,
                h_steam_pipe,
                h_steam_user,
                i_steam_apps,

                user_games: vec![],
            })
        }
    }

    pub fn populate_user_games(&mut self, list_to_check: Vec<SAMGame>) -> Result<(), SAMError> {
        for entry in list_to_check {
            let installed = unsafe {
                SteamAPI_ISteamApps_BIsSubscribedApp(self.i_steam_apps, entry.appid as AppId_t)
            };
            if installed {
                self.user_games.push(entry);
            }
        }

        Ok(())
    }

    pub fn get_user_games(&self) -> &Vec<SAMGame> {
        &self.user_games
    }
}

#[derive(Debug, Error)]
pub enum SAMError {
    #[error("there was an error in requesting the app list")]
    AppListRequestError,
    #[error("there was an error in deserializing the app list response: '{0}'")]
    AppListDeserializationError(String),

    #[error("there was an error when loading steamclient.{{so,dll}}: '{0}'")]
    ErrorLoadingSteamClientLib(String),
    #[error("there was an error when trying to get a pointer to CreateInterface: '{0}'")]
    ErrorGettingCreateInterface(String),
}
