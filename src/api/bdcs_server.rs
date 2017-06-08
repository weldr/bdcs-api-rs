//! BDCS Content Store server
//!
//! # Overview
//!
//! This module serves up the static bdcs ostree content store files
//!
//! It is accessed via the `/api/bdcs/<file>` route and will serve up any file under the path.
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

// A lot of the code generated via rocket uses pass-by-value that clippy
// disagrees with. Ignore these warnings.
#![cfg_attr(feature="cargo-clippy", allow(needless_pass_by_value))]

use std::path::{Path, PathBuf};

use rocket::response::NamedFile;
use rocket::State;

pub struct BDCSPath(pub String);

#[get("/<file..>")]
pub fn files(file: PathBuf, bdcs_path: State<BDCSPath>) -> Option<NamedFile> {
    NamedFile::open(Path::new(&bdcs_path.0).join(file)).ok()
}
