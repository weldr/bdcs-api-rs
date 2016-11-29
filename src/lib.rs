//! BDCS Crate
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
//! ## Overview
//!
//! The bdcs crate contains 3 modules. The db module for operations related to
//! the sqlite metadata store, and the api module for handling requests to
//! the bdcs API Server. The config module only exports the BDCSConfig struct
//! which is used to pass configuration data to the API handlers.
//!

extern crate flate2;
extern crate glob;
extern crate hyper;
#[macro_use] extern crate nickel;
extern crate nickel_sqlite;
extern crate rusqlite;
extern crate rustc_serialize;
extern crate toml;

pub mod api;
pub mod db;
mod config;
pub use config::BDCSConfig;
