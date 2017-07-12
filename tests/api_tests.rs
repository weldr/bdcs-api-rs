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
#[macro_use] extern crate pretty_assertions;

extern crate rocket;
extern crate serde_json;
extern crate toml;

use std::fs::{File, remove_dir_all};
use std::io::Write;

use bdcs::{RocketToml, RocketConfig};
use bdcs::api::v0;
use bdcs::db::DBPool;
use bdcs::recipe::{self, RecipeRepo};
use rocket::http::{ContentType, Method, Status};
use rocket::testing::MockRequest;
use serde_json::Value;

const DB_PATH: &'static str = "./tests/metadata.db";
// XXX This path is REMOVED on each run.
const RECIPE_PATH: &'static str = "/var/tmp/bdcs-recipes-test/";

/// Use lazy_static to write the config once, at runtime.
struct TestFramework {
    initialized: bool,
    rocket: rocket::Rocket
}

impl TestFramework {
    fn setup() -> TestFramework {
        write_config();
        setup_repo();

        let db_pool = DBPool::new(DB_PATH);
        let recipe_repo  = RecipeRepo::new(RECIPE_PATH);

        // Mount the API and run a request against it
        let rocket = rocket::ignite().mount("/",
                                            routes![v0::test,
                                            v0::isos,
                                            v0::compose_types,
                                            v0::projects_list_default, v0::projects_list_filter,
                                            v0::projects_info,
                                            v0::projects_depsolve,
                                            v0::modules_info,
                                            v0::modules_list_noargs_default, v0::modules_list_noargs_filter,
                                            v0::recipes_list_default, v0::recipes_list_filter,
                                            v0::recipes_info,
                                            v0::recipes_freeze,
                                            v0::recipes_changes_default, v0::recipes_changes_filter,
                                            v0::recipes_diff,
                                            v0::recipes_new_json, v0::recipes_new_toml,
                                            v0::recipes_workspace_json, v0::recipes_workspace_toml,
                                            v0::recipes_delete,
                                            v0::recipes_undo,
                                            v0::recipes_depsolve,
                                            v0::recipes_tag])
                                    .manage(db_pool)
                                    .manage(recipe_repo);

        TestFramework {
            initialized: true,
            rocket:      rocket
        }
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
    // Ignore ENOENT, fail on anything else
    match remove_dir_all(RECIPE_PATH) {
        Ok(_)  => (),
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => (),
            _ => panic!("Unable to remove {}: {}", RECIPE_PATH, e)
        }
    };

    // Write out the config to a Rocket.toml (this is easier than using rocket::custom)
    let rocket_config = RocketToml {
        global: RocketConfig {
            address: "127.0.0.1".to_string(),
            port: 4000,
            bdcs_path: "".to_string(),
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


/// Setup the Recipe git repo and import example recipes into it.
fn setup_repo() {
    let repo = recipe::init_repo(RECIPE_PATH).unwrap();
    recipe::add_dir(&repo, "./examples/recipes/", "master", false).unwrap();
}

#[test]
fn test_v0_test() {
    assert_eq!(FRAMEWORK.initialized, true);
    let rocket = &FRAMEWORK.rocket;

    let mut req = MockRequest::new(Method::Get, "/test");
    let mut response = req.dispatch_with(rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some("API v0 test".to_string()));
}

#[test]
fn test_v0_isos() {
    assert_eq!(FRAMEWORK.initialized, true);
    let rocket = &FRAMEWORK.rocket;

    // v0_isos()
    let mut req = MockRequest::new(Method::Get, "/isos");
    let mut response = req.dispatch_with(rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some("Unimplemented".to_string()));
}

#[test]
fn test_v0_compose_types() {
    assert_eq!(FRAMEWORK.initialized, true);
    let rocket = &FRAMEWORK.rocket;

    // v0_compose_types()
    let expected = include_str!("results/v0/compose-types.json").trim_right();

    let mut req = MockRequest::new(Method::Get, "/compose/types");
    let mut response = req.dispatch_with(rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some(expected.to_string()));
}

#[test]
fn test_v0_projects_list() {
    assert_eq!(FRAMEWORK.initialized, true);
    let rocket = &FRAMEWORK.rocket;

    // v0_projects_list()
    let expected_default = include_str!("results/v0/projects-list.json").trim_right();
    let expected_filter = include_str!("results/v0/projects-list-filter.json").trim_right();

    let mut req = MockRequest::new(Method::Get, "/projects/list");
    let mut response = req.dispatch_with(rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some(expected_default.to_string()));

    let mut req = MockRequest::new(Method::Get, "/projects/list?offset=2&limit=2");
    let mut response = req.dispatch_with(rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some(expected_filter.to_string()));
}

// NOTE the minimal database doesn't depsolve, so this checks for an empty response
#[test]
fn test_projects_depsolve() {
    assert_eq!(FRAMEWORK.initialized, true);
    let rocket = &FRAMEWORK.rocket;

    // v0_projects_depsolve()
    let expected = include_str!("results/v0/projects-depsolve.json").trim_right();

    let mut req = MockRequest::new(Method::Get, "/projects/depsolve/bash");
    let mut response = req.dispatch_with(rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some(expected.to_string()));
}

#[test]
fn test_v0_projects_info() {
    assert_eq!(FRAMEWORK.initialized, true);
    let rocket = &FRAMEWORK.rocket;

    // v0_projects_info()
    let expected_default = include_str!("results/v0/projects-info.json").trim_right();

    let mut req = MockRequest::new(Method::Get, "/projects/info/bash");
    let mut response = req.dispatch_with(rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some(expected_default.to_string()));
}


#[test]
fn test_v0_modules_info() {
    assert_eq!(FRAMEWORK.initialized, true);
    let rocket = &FRAMEWORK.rocket;

    // v0_modules_info()
    let expected_default = include_str!("results/v0/modules-info.json").trim_right();

    let mut req = MockRequest::new(Method::Get, "/modules/info/basesystem");
    let mut response = req.dispatch_with(rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some(expected_default.to_string()));
}

#[test]
fn test_v0_modules_list_noargs() {
    assert_eq!(FRAMEWORK.initialized, true);
    let rocket = &FRAMEWORK.rocket;

    // v0_modules_list_noargs()
    let expected_default = include_str!("results/v0/modules-list.json").trim_right();
    let expected_filter = include_str!("results/v0/modules-list-filter.json").trim_right();

    let mut req = MockRequest::new(Method::Get, "/modules/list");
    let mut response = req.dispatch_with(rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some(expected_default.to_string()));

    let mut req = MockRequest::new(Method::Get, "/modules/list?offset=2&limit=2");
    let mut response = req.dispatch_with(rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some(expected_filter.to_string()));
}

#[test]
fn test_v0_recipes_info() {
    assert_eq!(FRAMEWORK.initialized, true);
    let rocket = &FRAMEWORK.rocket;

    // v0_recipes_info()
    let expected_default = include_str!("results/v0/recipes-info.json").trim_right();

    let mut req = MockRequest::new(Method::Get, "/recipes/info/jboss,kubernetes");
    let mut response = req.dispatch_with(rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some(expected_default.to_string()));
}

#[test]
fn test_v0_recipes_changes() {
    assert_eq!(FRAMEWORK.initialized, true);
    let rocket = &FRAMEWORK.rocket;

    // v0_recipes_changes()
    let mut req = MockRequest::new(Method::Get, "/recipes/changes/atlas,kubernetes");
    let mut response = req.dispatch_with(rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap_or_default();
    let j: Value = serde_json::from_str(&body_str).unwrap();
    assert_eq!(j["recipes"][0]["name"], "atlas".to_string());
    assert_eq!(j["recipes"][0]["changes"][0]["message"], "Recipe atlas, version 0.0.1 saved".to_string());
    assert_eq!(j["recipes"][0]["total"], Value::from(1));
    assert_eq!(j["recipes"][1]["name"], "kubernetes".to_string());
    assert_eq!(j["recipes"][1]["changes"][0]["message"], "Recipe kubernetes, version 0.0.1 saved".to_string());
    assert_eq!(j["recipes"][1]["total"], Value::from(1));

    let mut req = MockRequest::new(Method::Get, "/recipes/changes/atlas?limit=1");
    let mut response = req.dispatch_with(rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap_or_default();
    let j: Value = serde_json::from_str(&body_str).unwrap();
    assert_eq!(j["recipes"][0]["name"], "atlas".to_string());
    assert_eq!(j["recipes"][0]["changes"][0]["message"], "Recipe atlas, version 0.0.1 saved".to_string());

    let mut req = MockRequest::new(Method::Get, "/recipes/changes/atlas?offset=1&limit=1");
    let mut response = req.dispatch_with(rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap_or_default();
    let j: Value = serde_json::from_str(&body_str).unwrap();
    assert_eq!(j["recipes"][0]["name"], "atlas".to_string());
    assert_eq!(j["recipes"][0]["changes"], Value::Array(vec![]));
    assert_eq!(j["recipes"][0]["total"], Value::from(1));
}

// NOTE the minimal database doesn't depsolve, so this checks for an empty response
#[test]
fn test_recipes_depsolve() {
    assert_eq!(FRAMEWORK.initialized, true);
    let rocket = &FRAMEWORK.rocket;

    // v0_recipes_depsolve()
    let expected = include_str!("results/v0/recipes-depsolve.json").trim_right();

    let mut req = MockRequest::new(Method::Get, "/recipes/depsolve/kubernetes");
    let mut response = req.dispatch_with(rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some(expected.to_string()));
}

// NOTE the minimal database doesn't depsolve, so this checks for an empty response
#[test]
fn test_recipes_freeze() {
    assert_eq!(FRAMEWORK.initialized, true);
    let rocket = &FRAMEWORK.rocket;

    // v0_recipes_freeze()
    let expected = include_str!("results/v0/recipes-freeze.json").trim_right();

    let mut req = MockRequest::new(Method::Get, "/recipes/freeze/http-server");
    let mut response = req.dispatch_with(rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some(expected.to_string()));
}

#[test]
fn test_v0_recipes() {
    // NOTE All the recipe tests need to be in the same thread, otherwise they will
    // interfere with each other
    assert_eq!(FRAMEWORK.initialized, true);
    let rocket = &FRAMEWORK.rocket;

    // v0_recipes_list()
    // TODO Copy ./examples/recipes/ to a temporary directory

    let expected_default = include_str!("results/v0/recipes-list.json").trim_right();
    let expected_filter = include_str!("results/v0/recipes-list-filter.json").trim_right();

    let mut req = MockRequest::new(Method::Get, "/recipes/list/");
    let mut response = req.dispatch_with(rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some(expected_default.to_string()));

    let mut req = MockRequest::new(Method::Get, "/recipes/list?limit=2");
    let mut response = req.dispatch_with(rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some(expected_filter.to_string()));

    // v0_recipes_new()
    let recipe_json = include_str!("results/v0/recipes-new.json").trim_right();

    let mut req = MockRequest::new(Method::Post, "/recipes/new")
                    .header(ContentType::JSON)
                    .body(recipe_json);
    let mut response = req.dispatch_with(rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some("{\"status\":true}".to_string()));

    // v0_recipes_delete
    // Delete the test recipe created above
    let mut req = MockRequest::new(Method::Delete, "/recipes/delete/recipe-test");
    let mut response = req.dispatch_with(rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some("{\"status\":true}".to_string()));

    // v0_recipes_new_toml()
    // Update the example http-server recipe with some changes.
    let recipe_toml = include_str!("results/v0/http-server.toml").trim_right();

    let mut req = MockRequest::new(Method::Post, "/recipes/new")
                    .header(ContentType::new("text", "x-toml"))
                    .body(recipe_toml);
    let mut response = req.dispatch_with(rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some("{\"status\":true}".to_string()));

    // v0_recipes_diff()
    // Need the commit id from the change to http-server for the next test
    let mut req = MockRequest::new(Method::Get, "/recipes/changes/http-server");
    let mut response = req.dispatch_with(rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap_or_default();
    let j: Value = serde_json::from_str(&body_str).unwrap();
    assert_eq!(j["recipes"][0]["name"], "http-server".to_string());
    assert_eq!(j["recipes"][0]["changes"][1]["message"], "Recipe http-server, version 0.0.1 saved".to_string());

    // Convert serde::Value to a &str
    let commit_id = match j["recipes"][0]["changes"][1]["commit"].as_str() {
        Some(str) => str,
        None => ""
    };

    // Check the diff
    let expected_diff = include_str!("results/v0/recipes-diff.json").trim_right();

    let mut req = MockRequest::new(Method::Get, format!("/recipes/diff/http-server/{}/NEWEST", commit_id));
    let mut response = req.dispatch_with(rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some(expected_diff.to_string()));

    // First write some changes to the recipe
    let recipe_json = include_str!("results/v0/recipes-new-v2.json").trim_right();

    // v0_recipes_workspace
    // Write the new recipe to the workspace first, confirm recipes/info returns changed:true
    let mut req = MockRequest::new(Method::Post, "/recipes/workspace")
                    .header(ContentType::JSON)
                    .body(recipe_json);
    let mut response = req.dispatch_with(rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some("{\"status\":true}".to_string()));

    let mut req = MockRequest::new(Method::Get, "/recipes/info/recipe-test");
    let mut response = req.dispatch_with(rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap_or_default();
    let j: Value = serde_json::from_str(&body_str).unwrap();
    assert_eq!(j["changes"][0]["name"], "recipe-test".to_string());
    assert_eq!(j["changes"][0]["changed"], Value::Bool(true));

    // Test workspace changes booleans
    let mut req = MockRequest::new(Method::Post, "/recipes/new")
                    .header(ContentType::JSON)
                    .body(recipe_json);
    let mut response = req.dispatch_with(rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some("{\"status\":true}".to_string()));

    // Confirm that info now shows changed:false
    let mut req = MockRequest::new(Method::Get, "/recipes/info/recipe-test");
    let mut response = req.dispatch_with(rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap_or_default();
    let j: Value = serde_json::from_str(&body_str).unwrap();
    assert_eq!(j["changes"][0]["name"], "recipe-test".to_string());
    assert_eq!(j["changes"][0]["changed"], Value::Bool(false));

    // Tag the latest recipe commit (it will be revision 1)
    let mut req = MockRequest::new(Method::Post, "/recipes/tag/recipe-test");
    let mut response = req.dispatch_with(rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some("{\"status\":true}".to_string()));

    // Write some new changes to the workspace
    let recipe_json = include_str!("results/v0/recipes-new-v3.json").trim_right();

    let mut req = MockRequest::new(Method::Post, "/recipes/workspace")
                    .header(ContentType::JSON)
                    .body(recipe_json);
    let mut response = req.dispatch_with(rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some("{\"status\":true}".to_string()));

    // Check the diff between the NEWEST and WORKSPACE
    let expected_diff = include_str!("results/v0/recipes-diff-workspace.json").trim_right();

    let mut req = MockRequest::new(Method::Get, "/recipes/diff/recipe-test/NEWEST/WORKSPACE");
    let mut response = req.dispatch_with(rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some(expected_diff.to_string()));

    // Get the original commit
    let mut req = MockRequest::new(Method::Get, "/recipes/changes/recipe-test");
    let mut response = req.dispatch_with(rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap_or_default();
    let j: Value = serde_json::from_str(&body_str).unwrap();
    assert_eq!(j["recipes"][0]["name"], "recipe-test".to_string());
    assert_eq!(j["recipes"][0]["changes"][0]["revision"], 1);

    // Convert serde::Value to a &str
    let commit_id = match j["recipes"][0]["changes"][1]["commit"].as_str() {
        Some(str) => str,
        None => ""
    };

    // Undo the change
    let mut req = MockRequest::new(Method::Post, format!("/recipes/undo/recipe-test/{}", commit_id));
    let mut response = req.dispatch_with(rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string());
    assert_eq!(body_str, Some("{\"status\":true}".to_string()));

    // Confirm that info has reverted to the previous commit and has no revision
    let mut req = MockRequest::new(Method::Get, "/recipes/info/recipe-test");
    let mut response = req.dispatch_with(rocket);

    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap_or_default();
    let j: Value = serde_json::from_str(&body_str).unwrap();
    assert_eq!(j["recipes"][0]["name"], "recipe-test".to_string());
    assert_eq!(j["recipes"][0]["version"], "0.3.12".to_string());
    assert_eq!(j["recipes"][0]["changes"][0]["revision"], Value::Null);
}
