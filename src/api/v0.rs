//! BDCS API handlers version 0
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
//!
//! # TODO
//!
//!  * Implement generic gzip handling for all responses.
//!  * Handle Authentication, similar to the [example here.](https://auth0.com/blog/build-an-api-in-rust-with-jwt-authentication-using-nickelrs/)
//!
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use glob::glob;
use rocket::config;
use rocket::http::Status;
use rocket::request::FromParam;
use rocket_contrib::JSON;
use rustc_serialize::json::{self, ToJson, Json};
use toml;

// bdcs database functions
use db::*;
use api::DB;

// defaults for queries that return multiple responses
static OFFSET: i64 = 0;
static LIMIT: i64 = 20;


/// This is used for optional query parameters that filter the results
#[derive(FromForm)]
pub struct Filter {
    offset: Option<i64>,
    limit: Option<i64>
}

/// Test the connection to the API
///
/// # Returns
///
/// * a str
///
/// # Response
///
///  * Sends a string to the client - `API v0 test`
///
/// # TODO
///
/// * Change this to JSON and report the version number?
///
#[get("/test")]
pub fn test() -> &'static str {
   "API v0 test"
}


/// List the available isos
///
/// # Returns
///
/// * Unimplemented
///
/// # Response
///
/// * HTTP Error
///
/// This means that it will be implemented eventually, and is a valid path.
///
/// # TODO
///
/// * Change it to a meaningful error code and JSON response
///
#[get("/isos")]
pub fn isos<'r>() -> &'static str {
    "Unimplemented"
}


#[post("/compose")]
pub fn compose<'r>() -> &'static str {
    "Unimplemented"
}

#[post("/compose/cancel")]
pub fn compose_cancel<'r>() -> &'static str {
    "Unimplemented"
}

#[get("compose/status")]
pub fn compose_status<'r>() -> &'static str {
    "Unimplemented"
}

#[get("compose/status/<id>")]
pub fn compose_status_id<'r>(id: &str) -> &'static str {
    "Unimplemented"
}

#[get("compose/log/<kbytes>")]
pub fn compose_log<'r>(kbytes: usize) -> &'static str {
    "Unimplemented"
}


// /compose/types

#[derive(Serialize)]
pub struct ComposeTypes {
    name: String,
    enabled: bool
}

impl ComposeTypes {
    /// Create a new ComposeTypes struct
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the output type. eg. 'iso'
    /// * `enabled` - Whether or not that type is actually enabled.
    ///
    /// # Returns
    ///
    /// * A new [ComposeTypes](struct.ComposeTypes.html) struct
    ///
    fn new<S: Into<String>>(name: S, enabled: bool) -> ComposeTypes {
        ComposeTypes { name: name.into(), enabled: enabled }
    }
}

#[derive(Serialize)]
pub struct ComposeTypesResponse {
    types: Vec<ComposeTypes>
}

/// Return the compose types and whether or not they are currently supported
///
/// # Response
///
/// * JSON response with 'types' set to a list of {'name':value, 'enabled': true|false} entries.
///
/// # Panics
///
/// * Failure to serialize the response
///
/// # Examples
///
/// ```json
/// {"types":[{"enabled":true,"name":"iso"},{"enabled":false,"name":"disk-image"},{"enabled":false,"name":"fs-image"},{"enabled":false,"name":"ami"},{"enabled":false,"name":"tar"},{"enabled":false,"name":"live-pxe"},{"enabled":false,"name":"live-ostree"},{"enabled":false,"name":"oci"},{"enabled":false,"name":"vagrant"},{"enabled":false,"name":"qcow2"},{"enabled":false,"name":"vmdk"},{"enabled":false,"name":"vhdx"}]}
/// ```
#[get("/compose/types")]
pub fn compose_types() -> JSON<ComposeTypesResponse> {
    let mut types = Vec::new();
    types.push(ComposeTypes::new("iso", true));
    types.push(ComposeTypes::new("disk-image", false));
    types.push(ComposeTypes::new("fs-image", false));
    types.push(ComposeTypes::new("ami", false));
    types.push(ComposeTypes::new("tar", false));
    types.push(ComposeTypes::new("live-pxe", false));
    types.push(ComposeTypes::new("live-ostree", false));
    types.push(ComposeTypes::new("oci", false));
    types.push(ComposeTypes::new("vagrant", false));
    types.push(ComposeTypes::new("qcow2", false));
    types.push(ComposeTypes::new("vmdk", false));
    types.push(ComposeTypes::new("vhdx", false));

    JSON(ComposeTypesResponse { types: types })
}

// /projects/list

#[derive(Serialize)]
pub struct ProjectsResponse {
    projects: Vec<Projects>,
    offset: i64,
    limit: i64
}

/// Return a summary of available projects, filtered by limit and/or offset
#[get("/projects/list?<filter>")]
pub fn projects_list_filter(filter: Filter, db: DB) -> JSON<ProjectsResponse> {
    projects_list(db, filter.offset.unwrap_or(OFFSET), filter.limit.unwrap_or(LIMIT))
}

// This catches the path when no query string was passed
#[get("/projects/list", rank=2)]
pub fn projects_list_default(db: DB) -> JSON<ProjectsResponse> {
    projects_list(db, OFFSET, LIMIT)
}

/// Return detailed information about a list of package names
///
/// # Arguments
///
/// * `db` - Database pool connection
/// * `offset` - Number of results to skip before returning results. Default is 0.
/// * `limit` - Maximum number of results to return. It may return less. Default is 20.
///
/// # Response
///
/// * JSON response with a list of {'name': value, 'summary': value} entries inside {"projects":[]}
///
/// # Panics
///
/// * Failure to get a database connection
/// * Failure to serialize the response
///
/// # Examples
///
/// ```json
/// TODO
/// ```
///
fn projects_list(db: DB, offset: i64, limit: i64) -> JSON<ProjectsResponse> {
    let result = get_projects_name(db.conn(), "*", offset, limit);
    JSON(ProjectsResponse {
            projects: result.unwrap_or(vec![]),
            offset: offset,
            limit: limit
    })
}


// /projects/info/<projects>

#[derive(Debug,Serialize)]
pub struct ProjectsInfoResponse {
    projects: Vec<ProjectInfo>,
    offset:   i64,
    limit:    i64
}

/// Return detailed information about a list of project names filtered by limit and/or offset
#[get("/projects/info/<projects>?<filter>")]
pub fn projects_info_filter(projects: &str, filter: Filter, db: DB) -> JSON<ProjectsInfoResponse> {
    projects_info(projects, db, filter.offset.unwrap_or(OFFSET), filter.limit.unwrap_or(LIMIT))
}

// This catches the path when no query string was passed
#[get("/projects/info/<projects>", rank=2)]
pub fn projects_info_default(projects: &str, db: DB) -> JSON<ProjectsInfoResponse> {
    projects_info(projects, db, OFFSET, LIMIT)
}


/// Return detailed information about a list of project names
///
/// # Arguments
///
/// * `db` - Database pool connection
/// * `offset` - Number of results to skip before returning results. Default is 0.
/// * `limit` - Maximum number of results to return. It may return less. Default is 20.
/// * `projects` - Comma separated list of projects.
///
/// # Response
///
/// * JSON response with a list of {'name': value, 'summary': value} entries inside {"projects":[]}
///
/// # Panics
///
/// * Failure to get a database connection
/// * Failure to serialize the response
///
///
/// The response includes details about the project, the available builds for the project,
/// and the sources used for the builds.
///
/// # Examples
///
/// ```json
/// TODO
/// ```
///
fn projects_info(projects: &str, db: DB, offset: i64, limit: i64) -> JSON<ProjectsInfoResponse> {
    let projects: Vec<&str> = projects.split(",").collect();
    let result = get_projects_details(db.conn(), &projects, offset, limit);
    JSON(ProjectsInfoResponse {
            projects: result.unwrap_or(vec![]),
            offset: offset,
            limit: limit
    })
}


// /modules/info/<modules>

#[derive(Debug,Serialize)]
pub struct ModulesInfoResponse {
//    modules:  Vec<ModuleInfo>,
    offset:   i64,
    limit:    i64
}

/// Return detailed information about a list of module names filtered by limit and/or offset
#[get("/modules/info/<modules>?<filter>")]
pub fn modules_info_filter(modules: &str, filter: Filter, db: DB) -> JSON<ModulesInfoResponse> {
    modules_info(modules, db, filter.offset.unwrap_or(OFFSET), filter.limit.unwrap_or(LIMIT))
}

// This catches the path when no query string was passed
#[get("/modules/info/<modules>", rank=2)]
pub fn modules_info_default(modules: &str, db: DB) -> JSON<ModulesInfoResponse> {
    modules_info(modules, db, OFFSET, LIMIT)
}

/// Return detailed information about a list of module names
///
/// # Arguments
///
/// * `db` - Database pool connection
/// * `offset` - Number of results to skip before returning results. Default is 0.
/// * `limit` - Maximum number of results to return. It may return less. Default is 20.
/// * `modules` - Comma separated list of modules.
///
/// # Response
///
/// * JSON response with a list of {'name': value, 'summary': value} entries inside {"modules":[]}
///
/// # Panics
///
/// * Failure to get a database connection
/// * Failure to serialize the response
///
///
/// # Examples
///
/// ```json
/// TODO
/// ```
///
fn modules_info(modules: &str, db: DB, offset: i64, limit: i64) -> JSON<ModulesInfoResponse> {
    let modules: Vec<&str> = modules.split(",").collect();
//    let result = get_modules_details(db.conn(), &projects, offset, limit);
    JSON(ModulesInfoResponse {
//            modules: result.unwrap_or(vec![]),
            offset: offset,
            limit: limit
    })
}

// /modules/list/<modules>

#[derive(Debug,Serialize)]
pub struct ModulesListResponse {
    modules: Vec<Groups>,
    offset:  i64,
    limit:   i64
}

/// Return the name and group type for a list of module names filtered by limit and/or offset
#[get("/modules/list/<modules>?<filter>")]
pub fn modules_list_filter(modules: &str, filter: Filter, db: DB) -> JSON<ModulesListResponse> {
    modules_list(modules, db, filter.offset.unwrap_or(OFFSET), filter.limit.unwrap_or(LIMIT))
}

// This catches the path when no query string was passed
#[get("/modules/list/<modules>", rank=2)]
pub fn modules_list_default(modules: &str, db: DB) -> JSON<ModulesListResponse> {
    modules_list(modules, db, OFFSET, LIMIT)
}

// This catches the path when no modules are passed
#[get("/modules/list/?<filter>")]
pub fn modules_list_noargs_filter(filter: Filter, db: DB) -> JSON<ModulesListResponse> {
    modules_list("*", db, filter.offset.unwrap_or(OFFSET), filter.limit.unwrap_or(LIMIT))
}

#[get("/modules/list/", rank=2)]
pub fn modules_list_noargs_default(db: DB) -> JSON<ModulesListResponse> {
    modules_list("*", db, OFFSET, LIMIT)
}

/// List the available modules and their type
///
/// # Arguments
///
/// * `db` - Database pool connection
/// * `offset` - Number of results to skip before returning results. Default is 0.
/// * `limit` - Maximum number of results to return. It may return less. Default is 20.
/// * `modules` - Comma separated list of modules.
///
/// # Response
///
/// * JSON response with a list of {'name': value, 'group_type': value} entries inside {"modules":[]}
///
/// # Examples
///
/// ```json
/// TODO
/// ```
///
fn modules_list(mut modules: &str, db: DB, offset: i64, limit: i64) -> JSON<ModulesListResponse> {
    if modules.len() == 0 {
        modules = "*";
    }
    let modules: Vec<&str> = modules.split(",").collect();
    let mut result = get_groups_vec(db.conn(), &modules, offset, limit)
                     .unwrap_or(vec![]);
    result.sort();
    result.dedup();
    JSON(ModulesListResponse {
            modules: result,
            offset: offset,
            limit: limit
    })
}


// recipe related functions

// TODO These should go into a versioned recipe module

// NOTE We have to use rustc-serialize here because the toml package is also used by rocket
// and has already been imported using rustc-serialize. If rocket switches to using toml with
// serde then we can change.

// TODO some of the lower level operations here should be in a recipe library

// /recipes/list

#[derive(Debug, Serialize)]
pub struct RecipesListResponse {
    recipes: Vec<String>,
    offset:  i64,
    limit:   i64
}

/// Return a list of the available recipes
#[get("/recipes/list?<filter>")]
pub fn recipes_list_filter(filter: Filter) -> JSON<RecipesListResponse> {
    recipes_list(filter.offset.unwrap_or(OFFSET), filter.limit.unwrap_or(LIMIT))
}

#[get("/recipes/list", rank=2)]
pub fn recipes_list_default() -> JSON<RecipesListResponse> {
    recipes_list(OFFSET, LIMIT)
}

/// Return the list of available Recipes
///
/// # Arguments
///
/// * `offset` - Number of results to skip before returning results. Default is 0.
/// * `limit` - Maximum number of results to return. It may return less. Default is 20.
///
/// # Response
///
/// * JSON response with a list of recipe names - {'recipes': ["name1", ...]}
///
/// # Panics
///
/// * Failure to serialize the response
///
/// # Examples
///
/// ```json
/// {"recipes":["another","example","foo"]}
/// ```
///
fn recipes_list(offset: i64, limit: i64) -> JSON<RecipesListResponse> {
    // TODO This should be a per-user path
    let recipes_path = config::active()
                           .unwrap()
                           .get_str("recipe_path")
                           .unwrap_or("/var/tmp/recipes/");
    let recipes_glob = recipes_path.to_string() + "*.toml";

    let mut result = Vec::new();
    for path in glob(&recipes_glob).unwrap().filter_map(Result::ok) {
        // Parse the TOML recipe into a Recipe struct
        let mut input = String::new();
        let _ = File::open(path)
                    .unwrap()
                    .read_to_string(&mut input);
        let recipe: Recipe = toml::decode_str(&input).unwrap();
        result.push(recipe.name);
    }
    JSON(RecipesListResponse {
            recipes: result,
            offset: offset,
            limit: limit
    })
}

// /recipes/info/<names>

#[derive(Debug, Serialize)]
pub struct RecipesInfoResponse {
    recipes: HashMap<String, Recipe>,
    offset:  i64,
    limit:   i64
}

/// Composer Recipe
///
/// This is used to parse the full recipe's TOML, and to write a JSON representation of
/// the Recipe.
///
#[derive(Debug, RustcDecodable, RustcEncodable, Serialize)]
struct Recipe {
    name: String,
    description: Option<String>,
    modules: Option<Vec<Modules>>,
    packages: Option<Vec<Packages>>
}

/// Recipe Modules
///
/// This is used for the Recipe's `modules` section and can be serialized
/// to/from JSON and TOML.
#[derive(Debug, RustcDecodable, RustcEncodable, Serialize)]
struct Modules {
    name: String,
    version: Option<String>
}

/// Recipe Packages
///
/// This is used for the Recipe's `packages` section
#[derive(Debug, RustcDecodable, RustcEncodable, Serialize)]
struct Packages {
    name: String,
    version: Option<String>
}


/// Return a list of the available recipes
#[get("/recipes/info/<recipes>?<filter>")]
pub fn recipes_info_filter(recipes: &str, filter: Filter) -> JSON<RecipesInfoResponse> {
    recipes_info(recipes, filter.offset.unwrap_or(OFFSET), filter.limit.unwrap_or(LIMIT))
}

#[get("/recipes/info/<recipes>", rank=2)]
pub fn recipes_info_default(recipes: &str) -> JSON<RecipesInfoResponse> {
    recipes_info(recipes, OFFSET, LIMIT)
}

/// Return the contents of a recipe or list of recipes
///
/// # Arguments
///
/// * `offset` - Number of results to skip before returning results. Default is 0.
/// * `limit` - Maximum number of results to return. It may return less. Default is 20.
/// * `names` - Comma separated list of recipe names to return
///
/// # Response
///
/// * JSON response with recipe contents, using the recipe name(s) as keys
///
/// # Panics
///
/// * Failure to serialize the response
///
/// # Errors
///
/// # Examples
///
/// ```json
/// TODO
/// ```
///
fn recipes_info(mut recipe_names: &str, offset: i64, limit: i64) -> JSON<RecipesInfoResponse> {
    // TODO This should be a per-user path
    let recipe_path = config::active()
                          .unwrap()
                          .get_str("recipe_path")
                          .unwrap_or("/var/tmp/recipes/");

    let mut result: HashMap<String, Recipe> = HashMap::new();
    for name in recipe_names.split(",") {
        // TODO Filesystem Path needs to be sanitized!
        let path = format!("{}{}.toml", recipe_path, name.replace(" ", "-"));

        // Parse the TOML recipe into a Recipe struct
        let mut input = String::new();
        let _ = File::open(path)
                    .unwrap()
                    .read_to_string(&mut input);
        let recipe: Recipe = toml::decode_str(&input).unwrap();
        result.insert(recipe.name.clone(), recipe);

    }
    JSON(RecipesInfoResponse {
        recipes: result,
        offset:  offset,
        limit:   limit
    })
}


/*
-    server.get("/api/v0/dnf/transaction/:packages", unimplemented_v0);
-    server.get("/api/v0/dnf/info/:packages", dnf_info_packages_v0);
-
-    server.get("/api/v0/recipe/list", recipe_list_v0);
-    server.get("/api/v0/recipe/:names", get_recipe_v0);
-    server.post("/api/v0/recipe/:name", post_recipe_v0);

*/


