//! BDCS Configuration Data
//!
//! Copyright (C) 2016
//! Red Hat, Inc.  All rights reserved.
//!
//! This program is free software; you can redistribute it and/or modify
//! it under the terms of the GNU General Public License as published by
//! the Free Software Foundation; either version 2 of the License, or
//! (at your option) any later version.
//!
//! This program is distributed in the hope that it will be useful,
//! but WITHOUT ANY WARRANTY; without even the implied warranty of
//! MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//! GNU General Public License for more details.
//!
//! You should have received a copy of the GNU General Public License
//! along with this program.  If not, see <http://www.gnu.org/licenses/>.
//!
//! Note: This requires sqlite-devel, and openssl-devel on the host in order to build

#[derive(Clone)]
pub struct BDCSConfig {
    pub host: String,
    pub port: u16,
    pub db_path: String,
    pub static_files: String,
    pub recipe_path: String,
}
