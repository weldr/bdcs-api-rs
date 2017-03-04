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
#[macro_use] extern crate lazy_static;
extern crate rocket;
extern crate toml;

use std::fs::File;
use std::io::Write;

use bdcs::{RocketToml, RocketConfig};
use bdcs::api::mock;
use rocket::http::{Method, Status};
use rocket::testing::MockRequest;

const DB_PATH: &'static str = "./tests/metadata.db";
// XXX This path is REMOVED on each run.
const RECIPE_PATH: &'static str = "/var/tmp/bdcs-recipes-test/";


/// Use lazy_static to write the config once, at runtime.
struct TestFramework {
    initialized: bool
}

impl TestFramework {
    fn setup() -> TestFramework {
        write_config();

        TestFramework { initialized: true }
    }
}

lazy_static! {
    static ref FRAMEWORK: TestFramework = {
        TestFramework::setup()
    };
}

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
            db_path: DB_PATH.to_string(),
            recipe_path: RECIPE_PATH.to_string(),
            log_path: "/var/log/bdcs-api.log".to_string(),
            mockfiles_path: "./tests/results/v0/".to_string()

        }
    };

    // Write out a Rocket.toml config with [global] settings
    let rocket_toml = toml::to_string(&rocket_config).unwrap();
    File::create("Rocket.toml").unwrap()
        .write_all(rocket_toml.as_bytes()).unwrap();
}

#[test]
fn mock_route() {
    assert_eq!(FRAMEWORK.initialized, true);

    let expected_default = include_str!("results/v0/route.json");
    let expected_filter = include_str!("results/v0/route-filter.json");

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
    assert_eq!(FRAMEWORK.initialized, true);

    let expected_default = include_str!("results/v0/route-param.json");
    let expected_filter = include_str!("results/v0/route-param-filter.json");

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
    assert_eq!(FRAMEWORK.initialized, true);

    let expected_default = include_str!("results/v0/route-action.json");
    let expected_filter = include_str!("results/v0/route-action-filter.json");

    // Mount the API and run a request against it
    let rocket = rocket::ignite().mount("/", routes![mock::static_route_action, mock::static_route_action_filter]);

    let mut req = MockRequest::new(Method::Get, "/recipes/info/jboss");
    let mut response = req.dispatch_with(&rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some(expected_default.to_string()));

    let mut req = MockRequest::new(Method::Get, "/recipes/info/jboss?offset=10&limit=2");
    let mut response = req.dispatch_with(&rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some(expected_filter.to_string()));
}
