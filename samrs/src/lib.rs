/*
 * This file is part of samrs
 * Copyright (C) 2023 Jonni Liljamo <jonni@liljamo.com>
 *
 * Licensed under GPL-3.0
 * See LICENSE for licensing information.
 */

use steamworks::{Client, SResult, SingleClient};
use thiserror::Error;

pub mod applist;

pub struct SAM {
    pub client: Client,
    pub single_client: SingleClient,
}

impl SAM {
    pub fn init() -> SResult<Self> {
        let (client, single_client) = match Client::init() {
            Err(err) => {
                return Err(err);
            }
            Ok((client, single_client)) => (client, single_client),
        };

        Ok(Self {
            client,
            single_client,
        })
    }
}

#[derive(Debug, Error)]
pub enum SAMError {
    #[error("there was an error in requesting the app list")]
    AppListRequestError,
    #[error("there was an error in deserializing the app list response")]
    AppListDeserializationError(String),
}
