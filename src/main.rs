//! BDCS API Server
//!
// Copyright (C) 2016
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
//!
//! Note: This requires sqlite-devel, and openssl-devel on the host in order to build
#![feature(plugin)]
#![feature(proc_macro)]
#![plugin(rocket_codegen)]

extern crate bdcs;
extern crate clap;
extern crate rocket;
extern crate rusqlite;
extern crate rustc_serialize;
extern crate toml;


use std::fs::File;
use std::io::Write;

use bdcs::api::v0;
use clap::{Arg, App};

/// Configuration file used by Rocket
#[derive(RustcEncodable)]
struct RocketToml {
    global: RocketConfig
}

#[derive(RustcEncodable)]
struct RocketConfig {
    address: String,
    port: usize,
    db_path: String,
    recipe_path: String
}


/// Process Command Line Arguments and Serve the http API
fn main() {
    let matches = App::new("bdcs-api")
                            .about("A REST API on top of the BDCS")
                            .arg(Arg::with_name("host")
                                        .long("host")
                                        .value_name("HOSTNAME|IP")
                                        .help("Host or IP to bind to (127.0.0.1)")
                                        .takes_value(true))
                            .arg(Arg::with_name("port")
                                        .long("port")
                                        .value_name("PORT")
                                        .help("Port to bind to (4000)")
                                        .takes_value(true))
                            .arg(Arg::with_name("DB")
                                        .help("Path to the BDCS sqlite database")
                                        .required(true)
                                        .index(1))
                            .arg(Arg::with_name("RECIPES")
                                        .help("Path to the recipes")
                                        .required(true)
                                        .index(2))
                        .get_matches();

    // Write out the config to a Rocket.toml (this is easier than using rocket::custom)
    let rocket_config = RocketToml {
        global: RocketConfig {
            address: matches.value_of("host").unwrap_or("127.0.0.1").to_string(),
            port: matches.value_of("port").unwrap_or("").parse().unwrap_or(4000),
            db_path: matches.value_of("DB").unwrap().to_string(),
            recipe_path: matches.value_of("RECIPES").unwrap().to_string()
        }
    };

    // Write out a Rocket.toml config with [global] settings
    let rocket_toml = toml::encode(&rocket_config);
    File::create("Rocket.toml").unwrap()
        .write_all(toml::encode_str(&rocket_toml).as_bytes()).unwrap();

    rocket::ignite()
        .mount("/api/v0/", routes![v0::test, v0::isos, v0::compose, v0::compose_types, v0::compose_cancel,
                                   v0::compose_status, v0::compose_status_id, v0::compose_log,
                                   v0::projects_list_default, v0::projects_list_filter,
                                   v0::projects_info_default, v0::projects_info_filter,
                                   v0::modules_info_default, v0::modules_info_filter,
                                   v0::modules_list_default, v0::modules_list_filter,
                                   v0::modules_list_noargs_default, v0::modules_list_noargs_filter,
                                   v0::recipes_list_default, v0::recipes_list_filter,
                                   v0::recipes_info_default, v0::recipes_info_filter,
                                   v0::recipes_new])
        .launch();
}
