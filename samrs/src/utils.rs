/*
 * This file is part of samrs
 * Copyright (C) 2023 Jonni Liljamo <jonni@liljamo.com>
 *
 * Licensed under GPL-3.0
 * See LICENSE for licensing information.
 */

use crate::SAMError;

pub fn find_steam_installation_path() -> Result<String, SAMError> {
    // TODO: implement
    Ok(String::from("/home/skye/.local/share/Steam"))
}
