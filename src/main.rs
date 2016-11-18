//! BDCS API http Framework
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

#[macro_use] extern crate nickel;
extern crate nickel_sqlite;
extern crate r2d2;
extern crate r2d2_sqlite;
extern crate clap;

extern crate bdcs;

use clap::{Arg, App};

// Database Connection Pooling
use r2d2::{Pool, Config};
use r2d2_sqlite::SqliteConnectionManager;
use nickel_sqlite::{SqliteMiddleware};

// Web API Framework
use nickel::{Nickel, HttpRouter, StaticFilesHandler};

// API v0 functions
use bdcs::api::enable_cors;
use bdcs::api::v0::{unimplemented_v0, test_v0, compose_types_v0, dnf_info_packages_v0, project_list_v0, project_info_v0};

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
                                        .help("Port to bind to (8000)")
                                        .takes_value(true))
                            .arg(Arg::with_name("DB")
                                        .help("Path to the BDCS sqlite database")
                                        .required(true)
                                        .index(1))
                            .arg(Arg::with_name("STATIC")
                                        .help("Path to the static files")
                                        .required(true)
                                        .index(2))
                        .get_matches();

    let host = matches.value_of("host").unwrap_or("127.0.0.1");
    let port: u16 = matches.value_of("port").unwrap_or("").parse().unwrap_or(8000);
    let db_path = matches.value_of("DB").unwrap();
    let static_files = matches.value_of("STATIC").unwrap();

    let mut server = Nickel::new();

    // Use a pool of connections to the sqlite database
    let db_mgr = SqliteConnectionManager::new(db_path);
    let db_pool = Pool::new(Config::default(), db_mgr)
        .expect("Unable to initialize the connection pool.");
    server.utilize(SqliteMiddleware::with_pool(db_pool));

    server.utilize(enable_cors);
    server.utilize(StaticFilesHandler::new(static_files));

    server.get("/api/v0/test", test_v0);

    // Composer v0 API
    server.get("/api/v0/isos", unimplemented_v0);
    server.post("/api/v0/compose", unimplemented_v0);
    server.get("/api/v0/compose/status", unimplemented_v0);
    server.get("/api/v0/compose/status/:compose_id", unimplemented_v0);
    server.get("/api/v0/compose/types", compose_types_v0);
    server.get("/api/v0/compose/log/:kbytes", unimplemented_v0);
    server.post("/api/v0/compose/cancel", unimplemented_v0);

    server.get("/api/v0/dnf/transaction/:packages", unimplemented_v0);
    server.get("/api/v0/dnf/info/:packages", dnf_info_packages_v0);

    server.get("/api/v0/projects/list", project_list_v0);
    server.get("/api/v0/projects/info/:projects", project_info_v0);

    server.get("/api/v0/module/info/:modules", unimplemented_v0);
    // Is this first needed or will the 2nd just have an empty param?
    server.get("/api/v0/module/list", unimplemented_v0);
    server.get("/api/v0/module/list/:modules", unimplemented_v0);

    server.get("/api/v0/recipe/list", unimplemented_v0);
    server.get("/api/v0/recipe/:names", unimplemented_v0);
    server.post("/api/v0/recipe/:name", unimplemented_v0);

    server.listen(&(host, port)).unwrap();
}
