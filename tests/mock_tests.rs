// Copyright (C) 2017 Red Hat, Inc.
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
#![plugin(rocket_codegen)]

extern crate bdcs;
extern crate rocket;
extern crate toml;

use std::fs::{File, remove_file};
use std::io::{Read, Write};
use std::path::Path;

use bdcs::{RocketToml, RocketConfig};
use bdcs::api::mock;
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
fn mock_route() {
    let expected_default = include_str!("results/v0/route.json");
    let expected_filter = include_str!("results/v0/route-filter.json");

    write_config();

    // Mount the API and run a request against it
    let rocket = rocket::ignite().mount("/", routes![mock::static_route, mock::static_route_filter]);

    let mut req = MockRequest::new(Method::Get, "/route");
    let mut response = req.dispatch_with(&rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some(expected_default.to_string()));

    let mut req = MockRequest::new(Method::Get, "/route?offset=10&limit=2");
    let mut response = req.dispatch_with(&rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some(expected_filter.to_string()));
}

#[test]
fn mock_route_param() {
    let expected_default = include_str!("results/v0/route-param.json");
    let expected_filter = include_str!("results/v0/route-param-filter.json");

    write_config();

    // Mount the API and run a request against it
    let rocket = rocket::ignite().mount("/", routes![mock::static_route_param, mock::static_route_param_filter]);

    let mut req = MockRequest::new(Method::Get, "/route/param");
    let mut response = req.dispatch_with(&rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some(expected_default.to_string()));

    let mut req = MockRequest::new(Method::Get, "/route/param?offset=10&limit=2");
    let mut response = req.dispatch_with(&rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some(expected_filter.to_string()));
}

#[test]
fn mock_route_action() {
    let expected_default = include_str!("results/v0/route-action.json");
    let expected_filter = include_str!("results/v0/route-action-filter.json");

    write_config();

    // Mount the API and run a request against it
    let rocket = rocket::ignite().mount("/", routes![mock::static_route_action, mock::static_route_action_filter]);

    let mut req = MockRequest::new(Method::Get, "/recipes/info/http-server");
    let mut response = req.dispatch_with(&rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some(expected_default.to_string()));

    let mut req = MockRequest::new(Method::Get, "/recipes/info/http-server?offset=10&limit=2");
    let mut response = req.dispatch_with(&rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some(expected_filter.to_string()));
}

