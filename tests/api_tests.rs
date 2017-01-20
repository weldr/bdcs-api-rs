// Copyright (C) 2017
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
#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate bdcs;
extern crate rocket;
extern crate toml;

use std::fs::{File, remove_file};
use std::io::{Read, Write};
use std::path::Path;

use bdcs::{RocketToml, RocketConfig};
use bdcs::api::v0;
use rocket::config;
use rocket::http::{ContentType, Method, Status};
use rocket::testing::MockRequest;


/// Write Rocket.toml
///
/// The tests need access to a directory for recipes and a copy of the BDCS database
/// They cannot be passed on the cmdline, so for now they are hard-coded here.
///
/// # TODO
///
/// Setup the test environment properly.
///
fn write_config() {
    // Write out the config to a Rocket.toml (this is easier than using rocket::custom)
    let rocket_config = RocketToml {
        global: RocketConfig {
            address: "127.0.0.1".to_string(),
            port: 4000,
            db_path: "./metadata.db".to_string(),
            recipe_path: "/var/tmp/recipes/".to_string(),
            log_path: "/var/log/bdcs-api.log".to_string(),
            mockfiles_path: "./tests/results/v0/".to_string()

        }
    };

    // Write out a Rocket.toml config with [global] settings
    let rocket_toml = toml::encode(&rocket_config);
    File::create("Rocket.toml").unwrap()
        .write_all(toml::encode_str(&rocket_toml).as_bytes()).unwrap();
}

#[test]
fn it_works() {
        assert_eq!(true, true);
}

#[test]
fn v0_test() {
    write_config();

    // Mount the API and run a request against it
    let rocket = rocket::ignite().mount("/", routes![v0::test]);
    let mut req = MockRequest::new(Method::Get, "/test");
    let mut response = req.dispatch_with(&rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some("API v0 test".to_string()));
}

#[test]
fn v0_isos() {
    write_config();

    // Mount the API and run a request against it
    let rocket = rocket::ignite().mount("/", routes![v0::isos]);
    let mut req = MockRequest::new(Method::Get, "/isos");
    let mut response = req.dispatch_with(&rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some("Unimplemented".to_string()));
}

#[test]
fn v0_compose_types() {
    let expected = include_str!("results/v0/compose-types.json");

    write_config();

    // Mount the API and run a request against it
    let rocket = rocket::ignite().mount("/", routes![v0::compose_types]);
    let mut req = MockRequest::new(Method::Get, "/compose/types");
    let mut response = req.dispatch_with(&rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some(expected.to_string()));
}

#[test]
fn v0_projects_list() {
    let expected_default = include_str!("results/v0/projects-list.json");
    let expected_filter = include_str!("results/v0/projects-list-filter.json");

    write_config();

    // Mount the API and run a request against it
    let rocket = rocket::ignite().mount("/", routes![v0::projects_list_default, v0::projects_list_filter]);

    let mut req = MockRequest::new(Method::Get, "/projects/list");
    let mut response = req.dispatch_with(&rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some(expected_default.to_string()));

    let mut req = MockRequest::new(Method::Get, "/projects/list?offset=10&limit=2");
    let mut response = req.dispatch_with(&rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some(expected_filter.to_string()));
}

/// Currently not implemented
#[ignore]
#[test]
fn v0_modules_info() {
    let expected_default = include_str!("results/v0/modules-info.json");
    let expected_filter = include_str!("results/v0/modules-info-filter.json");

    write_config();

    // Mount the API and run a request against it
    let rocket = rocket::ignite().mount("/", routes![v0::modules_info_default, v0::modules_info_filter]);

    let mut req = MockRequest::new(Method::Get, "/modules/info/lorax");
    let mut response = req.dispatch_with(&rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some(expected_default.to_string()));

    let mut req = MockRequest::new(Method::Get, "/modules/info/lorax?limit=10");
    let mut response = req.dispatch_with(&rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some(expected_filter.to_string()));
}

#[test]
fn v0_modules_list_noargs() {
    let expected_default = include_str!("results/v0/modules-list.json");
    let expected_filter = include_str!("results/v0/modules-list-filter.json");

    write_config();

    // Mount the API and run a request against it
    let rocket = rocket::ignite().mount("/", routes![v0::modules_list_noargs_default, v0::modules_list_noargs_filter]);

    let mut req = MockRequest::new(Method::Get, "/modules/list");
    let mut response = req.dispatch_with(&rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some(expected_default.to_string()));

    let mut req = MockRequest::new(Method::Get, "/modules/list?offset=15&limit=2");
    let mut response = req.dispatch_with(&rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some(expected_filter.to_string()));
}

#[test]
fn v0_recipes_list() {
    // TODO Copy ./examples/recipes/ to a temporary directory

    let expected_default = include_str!("results/v0/recipes-list.json");
    let expected_filter = include_str!("results/v0/recipes-list-filter.json");

    write_config();

    // Mount the API and run a request against it
    let rocket = rocket::ignite().mount("/", routes![v0::recipes_list_default, v0::recipes_list_filter]);

    let mut req = MockRequest::new(Method::Get, "/recipes/list/");
    let mut response = req.dispatch_with(&rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some(expected_default.to_string()));

    let mut req = MockRequest::new(Method::Get, "/recipes/list?limit=2");
    let mut response = req.dispatch_with(&rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some(expected_filter.to_string()));
}

#[test]
fn v0_recipes_info() {
    // TODO Copy ./examples/recipes/ to a temporary directory

    let expected_default = include_str!("results/v0/recipes-info.json");
    let expected_filter = include_str!("results/v0/recipes-info-filter.json");

    write_config();

    // Mount the API and run a request against it
    let rocket = rocket::ignite().mount("/", routes![v0::recipes_info_default, v0::recipes_info_filter]);

    let mut req = MockRequest::new(Method::Get, "/recipes/info/example,http-server,nfs-server");
    let mut response = req.dispatch_with(&rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some(expected_default.to_string()));

    let mut req = MockRequest::new(Method::Get, "/recipes/info/example,http-server,nfs-server?limit=2");
    let mut response = req.dispatch_with(&rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some(expected_filter.to_string()));
}

#[test]
fn v0_recipes_new() {
    let recipe_json = include_str!("results/v0/recipes-new.json");
    let recipe_toml = include_str!("results/v0/recipes-new.toml");

    write_config();

    // Mount the API and run a request against it
    let rocket = rocket::ignite().mount("/", routes![v0::recipes_new]);

    let recipe_path = config::active()
                          .unwrap()
                          .get_str("recipe_path")
                          .unwrap_or("/var/tmp/recipes/")
                          .to_string() + "recipe-test.toml";

    // Cleanup any previous test results
    let _ = remove_file(&recipe_path);

    let mut req = MockRequest::new(Method::Post, "/recipes/new")
                    .header(ContentType::JSON)
                    .body(recipe_json);
    let response = req.dispatch_with(&rocket);

    assert_eq!(response.status(), Status::Ok);

    assert_eq!(Path::new(&recipe_path).exists(), true);

    let mut file_toml = String::new();
    let _ = File::open(&recipe_path)
                .unwrap()
                .read_to_string(&mut file_toml);
    assert_eq!(file_toml, recipe_toml);

    // Cleanup the test file
    let _ = remove_file(&recipe_path);
}
