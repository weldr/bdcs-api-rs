//! BDCS Crate
//!
// Copyright (C) 2016-2017
// Red Hat, Inc.  All rights reserved.
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; either version 2 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.
//
//! ## Overview
//!
//! The bdcs crate contains 2 modules. The [db](db/index.html) module for operations related to the
//! sqlite metadata store, and the [api](api/index.html) module for handling requests to the bdcs
//! API Server.
//!
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
extern crate toml;


pub mod api;
pub mod db;
pub mod recipe;
