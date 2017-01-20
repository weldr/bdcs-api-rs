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
//! # Overview
//!
//! This module provides version 0 of the BDCS API handlers. They are used for things such as
//! listing projects, retrieving detailed information on projects and modules, manipulating recipe
//! files, etc.
//!
//! ## TODO
//!
//!  * Implement generic gzip handling for all responses.
//!  * Handle Authentication, similar to the [example here.](https://auth0.com/blog/build-an-api-in-rust-with-jwt-authentication-using-nickelrs/)
//!
use std::collections::HashMap;

use rocket::config;
use rocket::http::Status;
use rocket_contrib::JSON;

// bdcs database functions
use db::*;
use recipe::{self, Recipe};
use api::{CORS, DB, Filter, OFFSET, LIMIT};


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
pub fn test() -> CORS<&'static str> {
    info!("/test");
   CORS("API v0 test")
}


/// List the available isos
///
/// # Returns
///
/// * Unimplemented
///
/// # Response
///
/// * 'Unimplemented' string
///
/// This means that it will be implemented eventually, and is a valid path.
///
/// # TODO
///
/// * Change it to a meaningful error code and JSON response
///
#[get("/isos")]
pub fn isos<'r>() -> CORS<&'static str> {
    CORS("Unimplemented")
}


/// Start a compose
///
/// # Returns
///
/// * Unimplemented
///
/// # Response
///
/// * 'Unimplemented' string
///
/// This means that it will be implemented eventually, and is a valid path.
///
/// # TODO
///
/// * Change it to a meaningful error code and JSON response
/// * Return an id that can be used for cancel and status
///
#[post("/compose")]
pub fn compose<'r>() -> CORS<&'static str> {
    CORS("Unimplemented")
}

/// Cancel a compose
///
/// # Returns
///
/// * Unimplemented
///
/// # Response
///
/// * 'Unimplemented' string
///
/// This means that it will be implemented eventually, and is a valid path.
///
/// # TODO
///
/// * Change it to a meaningful error code and JSON response
/// * Pass it an id of a running compose
///
#[post("/compose/cancel")]
pub fn compose_cancel<'r>() -> CORS<&'static str> {
    CORS("Unimplemented")
}

/// Get the status of all composes
///
/// # Returns
///
/// * Unimplemented
///
/// # Response
///
/// * 'Unimplemented' string
///
/// This means that it will be implemented eventually, and is a valid path.
///
/// # TODO
///
/// * Change it to a meaningful error code and JSON response
///
#[get("compose/status")]
pub fn compose_status<'r>() -> CORS<&'static str> {
    CORS("Unimplemented")
}

/// Get the status of a specific compose
///
/// # Returns
///
/// * Unimplemented
///
/// # Response
///
/// * 'Unimplemented' string
///
/// This means that it will be implemented eventually, and is a valid path.
///
/// # TODO
///
/// * Change it to a meaningful error code and JSON response
///
#[get("compose/status/<id>")]
pub fn compose_status_id<'r>(id: &str) -> CORS<&'static str> {
    CORS("Unimplemented")
}

/// Get the logs from a running compose
///
/// # Returns
///
/// * Unimplemented
///
/// # Response
///
/// * 'Unimplemented' string
///
/// This means that it will be implemented eventually, and is a valid path.
///
/// # TODO
///
/// * Change it to a meaningful error code and JSON response
/// * Pass it the id of a running compose
///
#[get("compose/log/<kbytes>")]
pub fn compose_log<'r>(kbytes: usize) -> CORS<&'static str> {
    CORS("Unimplemented")
}


// /compose/types

/// Structure to hold the types of composes and whether or not they are actually available.
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

/// Hold the JSON response for /compose/types
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
pub fn compose_types() -> CORS<JSON<ComposeTypesResponse>> {
    info!("/compose/types");
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

    CORS(JSON(ComposeTypesResponse { types: types }))
}


// /projects/list

/// Hold the JSON response for /projects/list
#[derive(Serialize)]
pub struct ProjectsResponse {
    projects: Vec<Projects>,
    offset: i64,
    limit: i64
}

/// Handler for `/projects/list` with offset and limit arguments.
#[get("/projects/list?<filter>")]
pub fn projects_list_filter(filter: Filter, db: DB) -> CORS<JSON<ProjectsResponse>> {
    projects_list(db, filter.offset.unwrap_or(OFFSET), filter.limit.unwrap_or(LIMIT))
}


/// Handler for `/projects/list` without any arguments.
#[get("/projects/list", rank=2)]
pub fn projects_list_default(db: DB) -> CORS<JSON<ProjectsResponse>> {
    projects_list(db, OFFSET, LIMIT)
}

/// Return a summary of available projects, filtered by limit and/or offset
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
fn projects_list(db: DB, offset: i64, limit: i64) -> CORS<JSON<ProjectsResponse>> {
    info!("/projects/list"; "offset" => offset, "limit" => limit);
    let result = get_projects_name(db.conn(), "*", offset, limit);
    CORS(JSON(ProjectsResponse {
            projects: result.unwrap_or(vec![]),
            offset: offset,
            limit: limit
    }))
}


// /projects/info/<projects>

/// Hold the JSON response for /projects/info/
#[derive(Debug,Serialize)]
pub struct ProjectsInfoResponse {
    projects: Vec<ProjectInfo>,
    offset:   i64,
    limit:    i64
}

/// Handler for `/projects/info/` with offset and limit arguments.
#[get("/projects/info/<projects>?<filter>")]
pub fn projects_info_filter(projects: &str, filter: Filter, db: DB) -> CORS<JSON<ProjectsInfoResponse>> {
    projects_info(projects, db, filter.offset.unwrap_or(OFFSET), filter.limit.unwrap_or(LIMIT))
}

// This catches the path when no query string was passed
/// Handler for `/projects/list` without arguments.
#[get("/projects/info/<projects>", rank=2)]
pub fn projects_info_default(projects: &str, db: DB) -> CORS<JSON<ProjectsInfoResponse>> {
    projects_info(projects, db, OFFSET, LIMIT)
}


/// Return detailed information about a list of project names filtered by limit and/or offset
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
fn projects_info(projects: &str, db: DB, offset: i64, limit: i64) -> CORS<JSON<ProjectsInfoResponse>> {
    info!("/projects/info/"; "projects" => projects, "offset" => offset, "limit" => limit);
    let projects: Vec<&str> = projects.split(",").collect();
    let result = get_projects_details(db.conn(), &projects, offset, limit);
    CORS(JSON(ProjectsInfoResponse {
            projects: result.unwrap_or(vec![]),
            offset: offset,
            limit: limit
    }))
}


// /modules/info/<modules>

/// Hold the JSON response for /modules/info/
#[derive(Debug,Serialize)]
pub struct ModulesInfoResponse {
    modules:  Vec<GroupDeps>,
    offset:   i64,
    limit:    i64
}

/// Handler for `/modules/info/` with offset and limit arguments.
#[get("/modules/info/<modules>?<filter>")]
pub fn modules_info_filter(modules: &str, filter: Filter, db: DB) -> CORS<JSON<ModulesInfoResponse>> {
    modules_info(modules, db, filter.offset.unwrap_or(OFFSET), filter.limit.unwrap_or(LIMIT))
}

/// Handler for `/modules/info/` without arguments.
#[get("/modules/info/<modules>", rank=2)]
pub fn modules_info_default(modules: &str, db: DB) -> CORS<JSON<ModulesInfoResponse>> {
    modules_info(modules, db, OFFSET, LIMIT)
}

/// Return detailed information about a list of module names filtered by limit and/or offset
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
fn modules_info(modules: &str, db: DB, offset: i64, limit: i64) -> CORS<JSON<ModulesInfoResponse>> {
    info!("/modules/info/"; "modules" => modules, "offset" => offset, "limit" => limit);
    let modules: Vec<&str> = modules.split(",").collect();
    let result = get_groups_deps_vec(db.conn(), &modules, offset, limit);
    CORS(JSON(ModulesInfoResponse {
            modules: result.unwrap_or(vec![]),
            offset: offset,
            limit: limit
    }))
}

// /modules/list/<modules>

/// Hold the JSON response for /modules/list/
#[derive(Debug,Serialize)]
pub struct ModulesListResponse {
    modules: Vec<Groups>,
    offset:  i64,
    limit:   i64
}

/// Handler for `/modules/list/` with module names, offset, and limit arguments.
#[get("/modules/list/<modules>?<filter>")]
pub fn modules_list_filter(modules: &str, filter: Filter, db: DB) -> CORS<JSON<ModulesListResponse>> {
    modules_list(modules, db, filter.offset.unwrap_or(OFFSET), filter.limit.unwrap_or(LIMIT))
}

/// Handler for `/modules/list/` without arguments.
#[get("/modules/list/<modules>", rank=2)]
pub fn modules_list_default(modules: &str, db: DB) -> CORS<JSON<ModulesListResponse>> {
    modules_list(modules, db, OFFSET, LIMIT)
}

/// Handler for `/modules/list/` without module names, but with offset and limit arguments.
#[get("/modules/list/?<filter>")]
pub fn modules_list_noargs_filter(filter: Filter, db: DB) -> CORS<JSON<ModulesListResponse>> {
    modules_list("*", db, filter.offset.unwrap_or(OFFSET), filter.limit.unwrap_or(LIMIT))
}

/// Handler for `/modules/info/` without arguments.
#[get("/modules/list/", rank=2)]
pub fn modules_list_noargs_default(db: DB) -> CORS<JSON<ModulesListResponse>> {
    modules_list("*", db, OFFSET, LIMIT)
}

/// Return the name and group type for a list of module names filtered by limit and/or offset
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
fn modules_list(mut modules: &str, db: DB, offset: i64, limit: i64) -> CORS<JSON<ModulesListResponse>> {
    if modules.len() == 0 {
        modules = "*";
    }
    info!("/modules/list/"; "modules" => modules, "offset" => offset, "limit" => limit);
    let modules: Vec<&str> = modules.split(",").collect();
    let mut result = get_groups_vec(db.conn(), &modules, offset, limit)
                     .unwrap_or(vec![]);
    result.sort();
    result.dedup();
    CORS(JSON(ModulesListResponse {
            modules: result,
            offset: offset,
            limit: limit
    }))
}


// recipe related functions

// TODO These should go into a versioned recipe module

// NOTE We have to use rustc-serialize here because the toml package is also used by rocket
// and has already been imported using rustc-serialize. If rocket switches to using toml with
// serde then we can change.

// TODO some of the lower level operations here should be in a recipe library

// /recipes/list

/// Hold the JSON response for /recipes/list
#[derive(Debug, Serialize)]
pub struct RecipesListResponse {
    recipes: Vec<String>,
    offset:  i64,
    limit:   i64
}

/// Handler for `/recipes/list/` with offset and limit arguments.
#[get("/recipes/list?<filter>")]
pub fn recipes_list_filter(filter: Filter) -> CORS<JSON<RecipesListResponse>> {
    recipes_list(filter.offset.unwrap_or(OFFSET), filter.limit.unwrap_or(LIMIT))
}

/// Handler for `/recipes/list/` without arguments.
#[get("/recipes/list", rank=2)]
pub fn recipes_list_default() -> CORS<JSON<RecipesListResponse>> {
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
fn recipes_list(offset: i64, limit: i64) -> CORS<JSON<RecipesListResponse>> {
    info!("/recipes/list"; "offset" => offset, "limit" => limit);
    // TODO This should be a per-user path
    let recipes_path = config::active()
                           .unwrap()
                           .get_str("recipe_path")
                           .unwrap_or("/var/tmp/recipes/");

    let mut result = recipe::list(&recipes_path).unwrap_or(vec![]);
    result.sort();
    result.dedup();
    result.truncate(limit as usize);
    CORS(JSON(RecipesListResponse {
            recipes: result,
            offset: offset,
            limit: limit
    }))
}


// /recipes/info/<names>

/// Hold the JSON response for /recipes/info/
#[derive(Debug, Serialize)]
pub struct RecipesInfoResponse {
    recipes: Vec<Recipe>,
    offset:  i64,
    limit:   i64
}

/// Handler for `/recipes/info/` with offset and limit arguments.
#[get("/recipes/info/<recipes>?<filter>")]
pub fn recipes_info_filter(recipes: &str, filter: Filter) -> CORS<JSON<RecipesInfoResponse>> {
    recipes_info(recipes, filter.offset.unwrap_or(OFFSET), filter.limit.unwrap_or(LIMIT))
}

/// Handler for `/recipes/info/` without arguments.
#[get("/recipes/info/<recipes>", rank=2)]
pub fn recipes_info_default(recipes: &str) -> CORS<JSON<RecipesInfoResponse>> {
    recipes_info(recipes, OFFSET, LIMIT)
}

/// Return the contents of a recipe or list of recipes
///
/// # Arguments
///
/// * `offset` - Number of results to skip before returning results. Default is 0.
/// * `limit` - Maximum number of results to return. It may return less. Default is 20.
/// * `recipe_names` - Comma separated list of recipe names to return
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
fn recipes_info(recipe_names: &str, offset: i64, limit: i64) -> CORS<JSON<RecipesInfoResponse>> {
    info!("/recipes/info/"; "recipe_names" => recipe_names, "offset" => offset, "limit" => limit);
    // TODO This should be a per-user path
    let recipe_path = config::active()
                          .unwrap()
                          .get_str("recipe_path")
                          .unwrap_or("/var/tmp/recipes/");

    let mut result = Vec::new();
    for name in recipe_names.split(",").take(limit as usize) {
        // TODO Filesystem Path needs to be sanitized!
        let path = format!("{}{}.toml", recipe_path, name.replace(" ", "-"));
        let _ = recipe::read(&path).map(|recipe| {
            result.push(recipe);
        });
    }
    result.sort();
    result.dedup();
    result.truncate(limit as usize);
    CORS(JSON(RecipesInfoResponse {
        recipes: result,
        offset:  offset,
        limit:   limit
    }))
}

/// Hold the JSON response for /recipes/new/
#[derive(Debug, Serialize)]
pub struct RecipesNewResponse {
    status: bool
}

/// Save a new Recipe
///
/// # Arguments
///
/// * `recipe` - Recipe to save, in JSON format
///
/// # Response
///
/// * JSON response with recipe contents, using the recipe name(s) as keys
///
/// # Panics
///
/// * Failure to serialize the response
///
///
/// The body of the POST should be a valid Recipe in JSON format. If it cannot be parsed an
/// error 400 will be returned.
///
/// # Examples
///
/// ```json
/// {"name":"http-server","description":"An example http server","modules":[{"name":"fm-httpd","version":"23.*"},{"name":"fm-php","version":"11.6.*"}],"packages":[{"name":"tmux","version":"2.2"}]}
/// ```
#[post("/recipes/new/", format="application/json", data="<recipe>")]
pub fn recipes_new(recipe: JSON<Recipe>) -> CORS<JSON<RecipesNewResponse>> {
    info!("/recipes/new/"; "recipe.name" => recipe.name);
    // TODO This should be a per-user path
    let recipe_path = config::active()
                          .unwrap()
                          .get_str("recipe_path")
                          .unwrap_or("/var/tmp/recipes/");

    let status = recipe::write(&recipe_path, &recipe).unwrap_or(false);

    // TODO Return error information
    CORS(JSON(RecipesNewResponse {
            status: status
    }))
}
