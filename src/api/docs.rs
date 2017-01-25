//! BDCS Document server
//!
//! # Overview
//!
//! This module serves up the static documentation files created by running `cargo doc`
//!

// Copyright (C) 2016-2017 Red Hat, Inc.
//
// This file is part of bdcs-api-server.
//
// bdcs-api-server is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// bdcs-api-server is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with bdcs-api-server.  If not, see <http://www.gnu.org/licenses/>.

use std::path::{Path, PathBuf};

use rocket::response::{NamedFile, Redirect};

#[get("/")]
pub fn index() -> Redirect {
    // TODO Is there some way to make this relative to the mountpoint?
    Redirect::to("/api/docs/bdcs_api_server/index.html")
}

#[get("/<file..>")]
pub fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("target/doc/").join(file)).ok()
}
