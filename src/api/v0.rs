//! BDCS API handlers version 0
//!
//! # Overview
//!
//! This module provides version 0 of the BDCS API handlers. They are used for things such as
//! listing projects, retrieving detailed information on projects and modules, manipulating recipe
//! files, etc.
//!
//! # v0 API routes
//!
//! * `/api/v0/test`
//! * `/api/v0/isos`
//! * `/api/v0/compose`
//! * `/api/v0/compose/cancel`
//! * `/api/v0/compose/status`
//! * `/api/v0/compose/status/<id>`
//! * `/api/v0/compose/log/<kbytes>`
//! * `/api/v0/compose/types`
//!  - Return the types of images that can be created
//!  - [Example JSON](fn.compose_types.html#examples)
//! * `/api/v0/projects/list`
//!  - Return summaries about available projects
//!  - [Example JSON](fn.projects_list.html#examples)
//!  - [Optional filter parameters](../index.html#optional-filter-parameters)
//! * `/api/v0/projects/info/<projects>
//!  - Return detailed information about the project, all of its builds, and the sources of the
//!    builds.
//!  - [Example JSON](fn.projects_info.html#examples)
//!  - [Optional filter parameters](../index.html#optional-filter-parameters)
//! * `/api/v0/modules/list`
//!  - Return a list of available modules
//!  - [Example JSON](fn.modules_list.html#examples)
//!  - [Optional filter parameters](../index.html#optional-filter-parameters)
//! * `/api/v0/modules/info/<modules>
//!  - Return detailed information about a module.
//!  - [Example JSON](fn.modules_info.html#examples)
//!  - [Optional filter parameters](../index.html#optional-filter-parameters)
//! * `/api/v0/recipes/list`
//!  - List the names of the available recipes
//!  - [Example JSON](fn.recipes_list.html#examples)
//!  - [Optional filter parameters](../index.html#optional-filter-parameters)
//! * `/api/v0/recipes/info/<recipes>`
//!  - Return the contents of the recipe
//!  - [Example JSON](fn.recipes_info.html#examples)
//! * `/api/v0/recipes/depsolve/<recipes>`
//!  - Return the recipe and summary information about all of its modules and packages.
//!  - [Example JSON](fn.recipes_depsolve.html#examples)
//! * POST `/api/v0/recipes/new`
//!  - Create or update a recipe.
//!  - The body of the post is a JSON representation of the recipe, using the same format
//!    received by ``/api/v0/recipes/info/<recipes>`
//!  - [Example JSON](fn.recipes_new.html#examples)
//!
//!
//! ## TODO
//!
//!  * Implement generic gzip handling for all responses.
//!  * Handle Authentication, similar to the [example here.](https://auth0.com/blog/build-an-api-in-rust-with-jwt-authentication-using-nickelrs/)
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

use rocket::State;
use rocket_contrib::JSON;

// bdcs database functions
use db::*;
use recipe::{self, RecipeRepo, Recipe};
use api::{CORS, Filter, OFFSET, LIMIT};


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
#[get("/compose/status")]
pub fn compose_status() -> CORS<&'static str> {
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
#[get("/compose/status/<id>")]
pub fn compose_status_id(id: &str) -> CORS<&'static str> {
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
#[get("/compose/log/<kbytes>")]
pub fn compose_log(kbytes: usize) -> CORS<&'static str> {
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
///
/// # Examples
///
/// ```json
/// {
///     "types": [
///         {
///             "name": "iso",
///             "enabled": true
///         },
///         {
///             "name": "disk-image",
///             "enabled": false
///         },
///         {
///             "name": "fs-image",
///             "enabled": false
///         },
///         ...
///         }
///     ]
/// }
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
///
/// This calls [projects_list](fn.projects_list.html) with the optional `offset` and/or `limit`
/// values.
#[get("/projects/list?<filter>")]
pub fn projects_list_filter(filter: Filter, db: State<DBPool>) -> CORS<JSON<ProjectsResponse>> {
    projects_list(db, filter.offset.unwrap_or(OFFSET), filter.limit.unwrap_or(LIMIT))
}


/// Handler for `/projects/list` without any arguments.
///
/// This calls [projects_list](fn.projects_list.html) with the default `offset` and `limit` values.
#[get("/projects/list", rank=2)]
pub fn projects_list_default(db: State<DBPool>) -> CORS<JSON<ProjectsResponse>> {
    projects_list(db, OFFSET, LIMIT)
}

/// Return a summary of available projects, filtered by limit and/or offset
///
/// # Arguments
///
/// * `db` - Database pool
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
///
///
/// # Examples
///
/// ```json
/// {
///     "projects": [
///         {
///             "name": "basesystem",
///             "summary": "The skeleton package which defines a simple CentOS Linux system",
///             "description": "Basesystem defines the components of a basic CentOS Linux\nsystem (for example, the package installation order to use during\nbootstrapping). Basesystem should be in every installation of a system,\nand it should never be removed.",
///             "homepage": null,
///             "upstream_vcs": "UPSTREAM_VCS"
///         },
///         ...
///     ],
///     "offset": 0,
///     "limit": 20
/// }
/// ```
///
pub fn projects_list(db: State<DBPool>, offset: i64, limit: i64) -> CORS<JSON<ProjectsResponse>> {
    info!("/projects/list"; "offset" => offset, "limit" => limit);
    let result = get_projects_name(&db.conn(), "*", offset, limit);
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
///
/// This calls [projects_info](fn.projects_info.html) with the optional `offset` and/or `limit`
/// values.
#[get("/projects/info/<projects>?<filter>")]
pub fn projects_info_filter(projects: &str, filter: Filter, db: State<DBPool>) -> CORS<JSON<ProjectsInfoResponse>> {
    projects_info(projects, db, filter.offset.unwrap_or(OFFSET), filter.limit.unwrap_or(LIMIT))
}

/// Handler for `/projects/list` without arguments.
///
/// This calls [projects_info](fn.projects_info.html) with the default `offset` and `limit` values.
#[get("/projects/info/<projects>", rank=2)]
pub fn projects_info_default(projects: &str, db: State<DBPool>) -> CORS<JSON<ProjectsInfoResponse>> {
    projects_info(projects, db, OFFSET, LIMIT)
}


/// Return detailed information about a list of project names filtered by limit and/or offset
///
/// # Arguments
///
/// * `db` - Database pool
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
///
///
/// The response includes details about the project, the available builds for the project,
/// and the sources used for the builds.
///
///
/// # Examples
///
/// ```json
/// {
///     "projects": [
///         {
///             "name": "httpd",
///             "summary": "Apache HTTP Server",
///             "description": "The Apache HTTP Server is a powerful, efficient, and extensible\nweb server.",
///             "homepage": "http://httpd.apache.org/",
///             "upstream_vcs": "UPSTREAM_VCS",
///             "metadata": {},
///             "builds": [
///                 {
///                     "epoch": 0,
///                     "release": "45.el7.centos",
///                     "arch": "x86_64",
///                     "build_time": "2016-11-14T18:06:40",
///                     "changelog": "- Remove index.html, add centos-noindex.tar.gz\n- change vstring\n- change symlink for poweredby.png\n- update welcome.conf with proper aliases",
///                     "build_config_ref": "BUILD_CONFIG_REF",
///                     "build_env_ref": "BUILD_ENV_REF",
///                     "metadata": {
///                         "packageName": "httpd"
///                     },
///                     "source": {
///                         "license": "ASL 2.0",
///                         "version": "2.4.6",
///                         "source_ref": "SOURCE_REF",
///                         "metadata": {}
///                     }
///                 }
///             ]
///         }
///     ],
///     "offset": 0,
///     "limit": 20
/// }
/// ```
///
pub fn projects_info(projects: &str, db: State<DBPool>, offset: i64, limit: i64) -> CORS<JSON<ProjectsInfoResponse>> {
    info!("/projects/info/"; "projects" => projects, "offset" => offset, "limit" => limit);
    let projects: Vec<&str> = projects.split(",").collect();
    let result = get_projects_details(&db.conn(), &projects, offset, limit);
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
///
/// This calls [modules_info](fn.modules_info.html) with the optional `offset` and/or `limit`
/// values.
#[get("/modules/info/<modules>?<filter>")]
pub fn modules_info_filter(modules: &str, filter: Filter, db: State<DBPool>) -> CORS<JSON<ModulesInfoResponse>> {
    modules_info(modules, db, filter.offset.unwrap_or(OFFSET), filter.limit.unwrap_or(LIMIT))
}

/// Handler for `/modules/info/` without arguments.
///
/// This calls [modules_info](fn.modules_info.html) with the default `offset` and `limit` values.
#[get("/modules/info/<modules>", rank=2)]
pub fn modules_info_default(modules: &str, db: State<DBPool>) -> CORS<JSON<ModulesInfoResponse>> {
    modules_info(modules, db, OFFSET, LIMIT)
}

/// Return detailed information about a list of module names filtered by limit and/or offset
///
/// # Arguments
///
/// * `db` - Database pool
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
///
///
/// # Examples
///
/// ```json
/// {
///     "modules": [
///         {
///             "name": "httpd",
///             "summary": "Apache HTTP Server",
///             "description": "The Apache HTTP Server is a powerful, efficient, and extensible\nweb server.",
///             "homepage": "http://httpd.apache.org/",
///             "upstream_vcs": "UPSTREAM_VCS",
///             "projects": []
///         }
///     ],
///     "offset": 0,
///     "limit": 20
/// }
/// ```
///
pub fn modules_info(modules: &str, db: State<DBPool>, offset: i64, limit: i64) -> CORS<JSON<ModulesInfoResponse>> {
    info!("/modules/info/"; "modules" => modules, "offset" => offset, "limit" => limit);
    let modules: Vec<&str> = modules.split(",").collect();
    let result = get_groups_deps_vec(&db.conn(), &modules, offset, limit);
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
///
/// This calls [modules_list](fn.modules_list.html) with the optional `offset` and/or `limit`
/// values.
#[get("/modules/list/<modules>?<filter>")]
pub fn modules_list_filter(modules: &str, filter: Filter, db: State<DBPool>) -> CORS<JSON<ModulesListResponse>> {
    modules_list(modules, db, filter.offset.unwrap_or(OFFSET), filter.limit.unwrap_or(LIMIT))
}

/// Handler for `/modules/list/` without arguments.
///
/// This calls [modules_list](fn.modules_list.html) with the default `offset` and `limit` values.
#[get("/modules/list/<modules>", rank=2)]
pub fn modules_list_default(modules: &str, db: State<DBPool>) -> CORS<JSON<ModulesListResponse>> {
    modules_list(modules, db, OFFSET, LIMIT)
}

/// Handler for `/modules/list/` without module names, but with offset and limit arguments.
///
/// This calls [modules_list](fn.modules_list.html) with a wildcard name, `*`, and the optional
/// `offset` and/or `limit` values.
#[get("/modules/list?<filter>")]
pub fn modules_list_noargs_filter(filter: Filter, db: State<DBPool>) -> CORS<JSON<ModulesListResponse>> {
    modules_list("*", db, filter.offset.unwrap_or(OFFSET), filter.limit.unwrap_or(LIMIT))
}

/// Handler for `/modules/info/` without arguments.
///
/// This calls [modules_list](fn.modules_list.html) with a wildcard name, `*`, and the default
/// `offset` and `limit` values.
#[get("/modules/list", rank=2)]
pub fn modules_list_noargs_default(db: State<DBPool>) -> CORS<JSON<ModulesListResponse>> {
    modules_list("*", db, OFFSET, LIMIT)
}

/// Return the name and group type for a list of module names filtered by limit and/or offset
///
/// # Arguments
///
/// * `db` - Database pool
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
/// {
///     "modules": [
///         {
///             "name": "basesystem",
///             "group_type": "rpm"
///         },
///         {
///             "name": "bash",
///             "group_type": "rpm"
///         },
///         {
///             "name": "filesystem",
///             "group_type": "rpm"
///         },
///         {
///             "name": "httpd",
///             "group_type": "rpm"
///         }
///     ],
///     "offset": 0,
///     "limit": 20
/// }
/// ```
///
pub fn modules_list(mut modules: &str, db: State<DBPool>, offset: i64, limit: i64) -> CORS<JSON<ModulesListResponse>> {
    if modules.is_empty() {
        modules = "*";
    }
    info!("/modules/list/"; "modules" => modules, "offset" => offset, "limit" => limit);
    let modules: Vec<&str> = modules.split(",").collect();
    let mut result = get_groups_vec(&db.conn(), &modules, offset, limit)
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

// /recipes/list

/// Hold the JSON response for /recipes/list
#[derive(Debug, Serialize)]
pub struct RecipesListResponse {
    recipes: Vec<String>,
    offset:  i64,
    limit:   i64
}

/// Handler for `/recipes/list/` with offset and limit arguments.
///
/// This calls [recipes_list](fn.recipes_list.html) with the optional `offset` and/or `limit`
/// values.
#[get("/recipes/list?<filter>")]
pub fn recipes_list_filter(filter: Filter, repo: State<RecipeRepo>) -> CORS<JSON<RecipesListResponse>> {
    recipes_list(filter.offset.unwrap_or(OFFSET), filter.limit.unwrap_or(LIMIT), repo)
}

/// Handler for `/recipes/list/` without arguments.
///
/// This calls [recipes_list](fn.recipes_list.html) with the default `offset` and `limit` values.
#[get("/recipes/list", rank=2)]
pub fn recipes_list_default(repo: State<RecipeRepo>) -> CORS<JSON<RecipesListResponse>> {
    recipes_list(OFFSET, LIMIT, repo)
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
/// {
///     "recipes": [
///         "development",
///         "glusterfs",
///         "http-server",
///         "jboss",
///         "kubernetes",
///         "octave",
///     ],
///     "offset": 0,
///     "limit": 20
/// }
/// ```
///
pub fn recipes_list(offset: i64, limit: i64, repo: State<RecipeRepo>) -> CORS<JSON<RecipesListResponse>> {
    info!("/recipes/list"; "offset" => offset, "limit" => limit);
    // TODO Get the user's branch name. Use master for now.

    let mut result = recipe::list(&repo.repo(), "master", None).unwrap_or(vec![]);
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
///
/// This calls [recipes_info](fn.recipes_info.html) with the optional `offset` and/or `limit`
/// values.
#[get("/recipes/info/<recipes>?<filter>")]
pub fn recipes_info_filter(recipes: &str, filter: Filter, repo: State<RecipeRepo>) -> CORS<JSON<RecipesInfoResponse>> {
    recipes_info(recipes, filter.offset.unwrap_or(OFFSET), filter.limit.unwrap_or(LIMIT), repo)
}

/// Handler for `/recipes/info/` without arguments.
///
/// This calls [recipes_info](fn.recipes_info.html) with the default `offset` and `limit` values.
#[get("/recipes/info/<recipes>", rank=2)]
pub fn recipes_info_default(recipes: &str, repo: State<RecipeRepo>) -> CORS<JSON<RecipesInfoResponse>> {
    recipes_info(recipes, OFFSET, LIMIT, repo)
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
///
/// # Examples
///
/// ```json
/// {
///     "recipes": [
///         {
///             "name": "http-server",
///             "description": "An example http server with PHP and MySQL support.",
///             "modules": [
///                 {
///                     "name": "httpd",
///                     "version": "2.4.*"
///                 },
///                 {
///                     "name": "mod_auth_kerb",
///                     "version": "5.4"
///                 },
///                 {
///                     "name": "mod_ssl",
///                     "version": "2.4.*"
///                 },
///                 {
///                     "name": "php",
///                     "version": "5.4.*"
///                 },
///                 {
///                     "name": "php-mysql",
///                     "version": "5.4.*"
///                 }
///             ],
///             "packages": [
///                 {
///                     "name": "tmux",
///                     "version": "2.2"
///                 },
///                 {
///                     "name": "openssh-server",
///                     "version": "6.6.*"
///                 },
///                 {
///                     "name": "rsync",
///                     "version": "3.0.*"
///                 }
///             ]
///         }
///     ],
///     "offset": 0,
///     "limit": 20
/// }
/// ```
///
pub fn recipes_info(recipe_names: &str, offset: i64, limit: i64, repo: State<RecipeRepo>) -> CORS<JSON<RecipesInfoResponse>> {
    info!("/recipes/info/"; "recipe_names" => recipe_names, "offset" => offset, "limit" => limit);
    // TODO Get the user's branch name. Use master for now.

    let mut result = Vec::new();
    for name in recipe_names.split(",").take(limit as usize) {
        // TODO Filesystem Path needs to be sanitized!
        let _ = recipe::read(&repo.repo(), &name, "master", None).map(|recipe| {
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

/// The CORS system 'protects' the client via an OPTIONS request to make sure it is allowed
///
/// This returns an empty response, with the CORS headers set by [CORS](struct.CORS.html).
// Rocket has a collision with Diesel so uses route instead
//#[options("/recipes/new/")]
#[route(OPTIONS, "/recipes/new")]
pub fn options_recipes_new() -> CORS<&'static str> {
    CORS("")
}


/// Handler for `/recipes/new`
/// Save a new Recipe
///
/// # Arguments
///
/// * `recipe` - Recipe to save, in JSON format
///
/// # Response
///
/// * JSON response with "status" set to true or false.
///
///
/// The body of the POST should be a valid Recipe in JSON format. If it cannot be parsed an
/// error 400 will be returned.
///
/// # Examples
///
/// ## POST body
///
/// ```json
/// {
///     "name": "http-server",
///     "description": "An example http server with PHP and MySQL support.",
///     "modules": [
///         {
///             "name": "httpd",
///             "version": "2.4.*"
///         },
///         {
///             "name": "mod_auth_kerb",
///             "version": "5.4"
///         },
///         {
///             "name": "mod_ssl",
///             "version": "2.4.*"
///         },
///         {
///             "name": "php",
///             "version": "5.4.*"
///         },
///         {
///             "name": "php-mysql",
///             "version": "5.4.*"
///         }
///     ],
///     "packages": [
///         {
///             "name": "tmux",
///             "version": "2.2"
///         },
///         {
///             "name": "openssh-server",
///             "version": "6.6.*"
///         },
///         {
///             "name": "rsync",
///             "version": "3.0.*"
///         }
///     ]
/// }
/// ```
///
/// ## Response
///
/// ```json
/// {
///     "status": true
/// }
/// ```
#[post("/recipes/new", format="application/json", data="<recipe>")]
pub fn recipes_new(recipe: JSON<Recipe>, repo: State<RecipeRepo>) -> CORS<JSON<RecipesNewResponse>> {
    info!("/recipes/new/"; "recipe.name" => recipe.name);
    // TODO Get the user's branch name. Use master for now.

    let status = match recipe::write(&repo.repo(), &recipe, "master") {
        Ok(result) => result,
        Err(e) => {
            error!("recipes_new"; "recipe" => format!("{:?}", recipe), "error" => format!("{:?}", e));
            false
        }
    };

    // TODO Return error information
    CORS(JSON(RecipesNewResponse {
            status: status
    }))
}


/// A Recipe and its dependencies
#[derive(Debug, Serialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecipeDeps {
    recipe: Recipe,
    modules: Vec<GroupDeps>,
}


/// Hold the JSON response for /recipes/depsolve/
#[derive(Debug, Serialize)]
pub struct RecipesDepsolveResponse {
    recipes: Vec<RecipeDeps>
}

/// Return the contents of a recipe and its dependencies
///
/// # Arguments
///
/// * `recipe_names` - Comma separated list of recipe names to return
///
/// # Response
///
/// * JSON response like: {"recipes": [{"recipe": {RECIPE}, "modules": [DEPS]}]}
///   Where RECIPE is the same JSON you would get from a /recipes/info/ query
///   and DEPS are the same as what you would get from a /modules/info/ query.
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
/// ```
/// {
///     "recipes": [
///         {
///             "recipe": {
///             [SAME CONTENT AS /recipes/info]
///             },
///             "modules": [
///                 {
///                     "name": "httpd",
///                     "summary": "Apache HTTP Server",
///                     "description": "The Apache HTTP Server is a powerful, efficient, and extensible\nweb server.",
///                     "homepage": "http://httpd.apache.org/",
///                     "upstream_vcs": "UPSTREAM_VCS",
///                     "projects": []
///                 },
///                 {
///                     "name": "mod_auth_kerb",
///                     "summary": "Kerberos authentication module for HTTP",
///                     "description": "mod_auth_kerb is module for the Apache HTTP Server designed to\nprovide Kerberos authentication over HTTP.  The module supports the\nNegotiate authentication method, which performs full Kerberos\nauthentication based on ticket exchanges.",
///                     "homepage": "http://modauthkerb.sourceforge.net/",
///                     "upstream_vcs": "UPSTREAM_VCS",
///                     "projects": [
///                         {
///                             "name": "httpd",
///                             "summary": "Apache HTTP Server",
///                             "description": "The Apache HTTP Server is a powerful, efficient, and extensible\nweb server.",
///                             "homepage": "http://httpd.apache.org/",
///                             "upstream_vcs": "UPSTREAM_VCS"
///                         }
///                     ]
///                 },
///                 ...
///             ]
///         }
///     ]
/// }
///
#[get("/recipes/depsolve/<recipe_names>")]
pub fn recipes_depsolve(recipe_names: &str, db: State<DBPool>, repo: State<RecipeRepo>) -> CORS<JSON<RecipesDepsolveResponse>> {
    info!("/recipes/depsolve/"; "recipe_names" => recipe_names);
    // TODO Get the user's branch name. Use master for now.

    let mut result = Vec::new();
    for name in recipe_names.split(",") {
        let _ = recipe::read(&repo.repo(), &name, "master", None).map(|recipe| {
            let mut modules = Vec::new();
            for module in recipe.clone().modules {
                match get_group_deps(&db.conn(), &module.name, 0, i64::max_value()) {
                    Ok(r) => modules.push(r),
                    Err(_) => {}
                }
            }

            for package in recipe.clone().packages {
                match get_group_deps(&db.conn(), &package.name, 0, i64::max_value()) {
                    Ok(r) => modules.push(r),
                    Err(_) => {}
                }
            }
            modules.sort();
            modules.dedup();

            result.push(RecipeDeps {
                            recipe: recipe,
                            modules: modules
            });
        });
    }
    result.sort();
    CORS(JSON(RecipesDepsolveResponse {
            recipes: result
    }))
}
