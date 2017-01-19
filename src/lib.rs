//! BDCS Crate
//!
//! ## Overview
//!
//! The bdcs crate is the library used by the bdcs-api-server.
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

#![feature(plugin)]
#![feature(proc_macro)]
#![feature(custom_derive)]
#![plugin(rocket_codegen)]

extern crate glob;
extern crate hyper;
#[macro_use] extern crate lazy_static;
extern crate r2d2;
extern crate r2d2_sqlite;
extern crate rocket;
#[macro_use] extern crate rocket_contrib;
extern crate rusqlite;
extern crate rustc_serialize;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate slog;
#[macro_use] extern crate slog_scope;
extern crate toml;


pub mod api;
pub mod db;
pub mod recipe;

/// Configuration file used by Rocket
#[derive(Debug, RustcEncodable)]
pub struct RocketToml {
    pub global: RocketConfig
}

#[derive(Debug, RustcEncodable)]
pub struct RocketConfig {
    pub address: String,
    pub port: usize,
    pub db_path: String,
    pub recipe_path: String,
    pub log_path: String,
    pub mockfiles_path: String
}

/// Unit tests for bdcs
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(true, true);
    }
}
