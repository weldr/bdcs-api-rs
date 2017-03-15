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
//! * `/api/v0/projects/info/<projects>`
//!  - Return detailed information about the project, all of its builds, and the sources of the
//!    builds.
//!  - [Example JSON](fn.projects_info.html#examples)
//! * `/api/v0/projects/depsolve/<projects>`
//!  - Returns the dependencies for the listed projects
//!  - [Example JSON](fn.projects_depsolve.html#examples)
//! * `/api/v0/modules/list`
//!  - Return a list of available modules
//!  - [Example JSON](fn.modules_list.html#examples)
//!  - [Optional filter parameters](../index.html#optional-filter-parameters)
//! * `/api/v0/modules/info/<modules>`
//!  - Return detailed information about a module.
//!  - [Example JSON](fn.modules_info.html#examples)
//! * `/api/v0/recipes/list`
//!  - List the names of the available recipes
//!  - [Example JSON](fn.recipes_list.html#examples)
//!  - [Optional filter parameters](../index.html#optional-filter-parameters)
//! * `/api/v0/recipes/info/<recipes>`
//!  - Return the contents of the recipe.
//!  - [Example JSON](fn.recipes_info.html#examples)
//! * `/api/v0/recipes/changes/<recipes>`
//!  - Return the commit history of the recipes
//!  - [Example JSON](fn.recipes_changes.html#examples)
//!  - [Optional filter parameters](../index.html#optional-filter-parameters)
//! * `/api/v0/recipes/diff/<recipe>/<from_commit>/<to_commit>`
//!  - Return the diff between the two recipe commits. Set to_commit to NEWEST to use the newest commit.
//!  - [Example JSON](fn.recipes_diff.html#examples)
//! * `/api/v0/recipes/depsolve/<recipes>`
//!  - Return the recipe and summary information about all of its modules and packages.
//!  - [Example JSON](fn.recipes_depsolve.html#examples)
//! * POST `/api/v0/recipes/new`
//!  - Create or update a recipe.
//!  - The body of the post is a JSON representation of the recipe, using the same format
//!    received by `/api/v0/recipes/info/<recipes>`
//!  - [Example JSON](fn.recipes_new.html#examples)
//! * DELETE `/api/v0/recipes/delete/<recipe>`
//!  - Delete the named recipe from the repository
//!  - [Example JSON](fn.recipes_delete.html#examples)
//! * POST `/api/v0/recipes/undo/<recipe>/<commit>`
//!  - Revert a recipe to a previous commit
//!  - [Example JSON](fn.recipes_undo.html#examples)
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
use std::cmp;
use std::rc::Rc;

use rocket::State;
use rocket_contrib::JSON;
use rusqlite::Connection;

// bdcs database functions
use db::*;
use depclose::*;
use depsolve::*;
use recipe::{self, RecipeRepo, Recipe, RecipeCommit};
use api::{CORS, Filter, Format, OFFSET, LIMIT};
use api::toml::TOML;



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
    types.push(ComposeTypes::new("kickstart", true));
    types.push(ComposeTypes::new("ostree", false));
    types.push(ComposeTypes::new("iso", false));
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
    offset:   i64,
    limit:    i64,
    total:    i64
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
///     "limit": 20,
///     "total": 1
/// }
/// ```
///
pub fn projects_list(db: State<DBPool>, offset: i64, limit: i64) -> CORS<JSON<ProjectsResponse>> {
    info!("/projects/list"; "offset" => offset, "limit" => limit);
    let (total, projects) = match get_projects_name(&db.conn(), "*", offset, limit) {
        Ok((total, projects)) => (total, projects),
        Err(_) => (0, vec![])
    };

    CORS(JSON(ProjectsResponse {
            projects: projects,
            offset:   offset,
            limit:    limit,
            total:    total
    }))
}


// /projects/info/<projects>

/// Hold the JSON response for /projects/info/
#[derive(Debug,Serialize)]
pub struct ProjectsInfoResponse {
    projects: Vec<ProjectInfo>,
}

/// Handler for `/projects/info`
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
#[get("/projects/info/<projects>")]
pub fn projects_info(projects: &str, db: State<DBPool>) -> CORS<JSON<ProjectsInfoResponse>> {
    info!("/projects/info/"; "projects" => projects);
    let projects: Vec<&str> = projects.split(",").collect();
    let result = get_projects_details(&db.conn(), &projects);
    CORS(JSON(ProjectsInfoResponse {
            projects: result.unwrap_or(vec![]),
    }))
}


/// Hold the JSON response for /projects/depsolve/
#[derive(Debug,Serialize)]
pub struct ProjectsDepsolveResponse {
    projects: Vec<PackageNEVRA>
}

/// Handler for `/projects/depsolve`
/// Depsolve a list of package
///
/// # Arguments
///
/// * `projects` - Comma separated list of project names
/// * `db` - Database pool
///
/// # Response
///
/// * JSON response with a list of {'name': value, 'summary': value} entries inside {"projects":[]}
///
///
///
/// # Examples
///
/// ```json
/// {
///     "projects": [
///         {
///             "name": "acl",
///             "epoch": 0,
///             "version": "2.2.51",
///             "release": "12.el7",
///             "arch": "x86_64"
///         },
///         {
///             "name": "apr",
///             "epoch": 0,
///             "version": "1.4.8",
///             "release": "3.el7",
///             "arch": "x86_64"
///         },
///         ...
///     ]
/// }
/// ```
#[get("/projects/depsolve/<projects>")]
pub fn projects_depsolve(projects: &str, db: State<DBPool>) -> CORS<JSON<ProjectsDepsolveResponse>> {
    info!("/projects/depsolve/"; "projects" => projects);
    let projects: Vec<String> = projects.split(",").map(|s| String::from(s)).collect();

    let pkg_nevras = depsolve_helper(&db.conn(), &projects);

    CORS(JSON(ProjectsDepsolveResponse {
        projects: pkg_nevras
    }))
 }

fn depsolve_helper(conn: &Connection, projects: &Vec<String>) -> Vec<PackageNEVRA> {
    // depclose the given projects into a big ol' depexpr
    let depexpr = match close_dependencies(conn, &vec!(String::from("x86_64")), projects) {
        Ok(d) => d,
        Err(e) => {
            error!("close_dependencies"; "projects" => format!("{:?}", projects), "error" => e);
            return vec![];
        }
    };

    // Wrap the returned depexpression in the crud it needs
    let mut exprs = vec![Rc::new(DepCell::new(depexpr))];

    match solve_dependencies(conn, &mut exprs) {
        Ok(ids) => {
            let mut nevras = pkg_nevra_groups_vec(conn, &ids);
            nevras.sort();
            return nevras;
        },
        Err(e) => {
            error!("Error depsolving"; "pkgs" => format!("{:?}", projects), "error" => e);
            return vec![];
        }
    }
}


// /modules/info/<modules>

/// Module info and dependencies
#[derive(Debug, Serialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct ModuleInfoDeps {
    pub name:         String,
    pub summary:      String,
    pub description:  String,
    pub homepage:     Option<String>,
    pub upstream_vcs: String,
    pub dependencies: Vec<PackageNEVRA>
}

/// Hold the JSON response for /modules/info/
#[derive(Debug,Serialize)]
pub struct ModulesInfoResponse {
    modules:  Vec<ModuleInfoDeps>,
}

/// Handler for `/modules/info/` without arguments.
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
///             "dependencies": [
///                 {
///                     "name": "acl",
///                     "epoch": 0,
///                     "version": "2.2.51",
///                     "release": "12.el7",
///                     "arch": "x86_64"
///                 },
///                 {
///                     "name": "apr",
///                     "epoch": 0,
///                     "version": "1.4.8",
///                     "release": "3.el7",
///                     "arch": "x86_64"
///                 },
///                 ...
///             ]
///         }
///     ]
/// }
/// ```
///
#[get("/modules/info/<modules>")]
pub fn modules_info(modules: &str, db: State<DBPool>) -> CORS<JSON<ModulesInfoResponse>> {
    info!("/modules/info/"; "modules" => modules);
    let modules: Vec<String> = modules.split(",").map(|s| String::from(s)).collect();

    let mut result = Vec::new();
    for m in modules {
        match get_projects_name(&db.conn(), &m, 0, i64::max_value()) {
            Ok((1, p)) => {
                let deps = depsolve_helper(&db.conn(), &vec![m]);
                result.push(ModuleInfoDeps {
                    name:         p[0].name.clone(),
                    summary:      p[0].summary.clone(),
                    description:  p[0].description.clone(),
                    homepage:     p[0].homepage.clone(),
                    upstream_vcs: p[0].upstream_vcs.clone(),
                    dependencies: deps
                });
            }
            Ok((0,_)) => {}
            Ok((_,_)) => {}
            Err(e) => {
                error!("Error looking up module info"; "module" => m, "error" => format!("{:?}", e));
            }
        }
    }

    CORS(JSON(ModulesInfoResponse {
            modules: result,
    }))
}

// /modules/list/<modules>

/// Hold the JSON response for /modules/list/
#[derive(Debug,Serialize)]
pub struct ModulesListResponse {
    modules: Vec<Groups>,
    offset:  i64,
    limit:   i64,
    total:   i64
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
///     "limit": 20,
///     "total": 4
/// }
/// ```
///
pub fn modules_list(mut modules: &str, db: State<DBPool>, offset: i64, limit: i64) -> CORS<JSON<ModulesListResponse>> {
    if modules.is_empty() {
        modules = "*";
    }
    info!("/modules/list/"; "modules" => modules, "offset" => offset, "limit" => limit);
    let modules: Vec<&str> = modules.split(",").collect();
    let mut result = get_groups_vec(&db.conn(), &modules)
                     .unwrap_or(vec![]);
    result.sort();
    // Groups includes the unique id, so dedupe using the name.
    result.dedup_by(|a, b| a.name.eq(&b.name));
    let total = result.len() as i64;

    result = result.into_iter().skip(offset as usize).take(limit as usize).collect();
    CORS(JSON(ModulesListResponse {
            modules: result,
            offset:  offset,
            limit:   limit,
            total:   total
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
    limit:   i64,
    total:   i64
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
///     "limit": 20,
///     "total": 6
/// }
/// ```
///
pub fn recipes_list(offset: i64, limit: i64, repo: State<RecipeRepo>) -> CORS<JSON<RecipesListResponse>> {
    info!("/recipes/list"; "offset" => offset, "limit" => limit);
    // TODO Get the user's branch name. Use master for now.

    let mut result = recipe::list(&repo.repo(), "master", None).unwrap_or(vec![]);
    result.sort();
    result.dedup();
    let total = result.len() as i64;
    result.truncate(limit as usize);
    CORS(JSON(RecipesListResponse {
            recipes: result,
            offset:  offset,
            limit:   limit,
            total:   total
    }))
}


// /recipes/info/<names>

/// Hold the JSON response for /recipes/info/
#[derive(Debug, Serialize)]
pub struct RecipesInfoResponse {
    recipes: Vec<Recipe>,
}


/// Handler for `/recipes/info/` without arguments.
/// Return the contents of a recipe or list of recipes
///
/// # Arguments
///
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
///             "version": "0.0.1",
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
/// }
/// ```
///
#[get("/recipes/info/<recipe_names>")]
pub fn recipes_info(recipe_names: &str, repo: State<RecipeRepo>) -> CORS<JSON<RecipesInfoResponse>> {
    info!("/recipes/info/ (JSON)"; "recipe_names" => recipe_names);
    // TODO Get the user's branch name. Use master for now.

    let mut result = Vec::new();
    for name in recipe_names.split(",") {
        let _ = recipe::read(&repo.repo(), &name, "master", None).map(|recipe| {
            result.push(recipe);
        });
    }
    result.sort();
    result.dedup();

    CORS(JSON(RecipesInfoResponse {
        recipes: result,
    }))
}

/// Return the requested recipe as TOML
/// Note that this only supports 1 recipe at a time
///
/// The request should be: `/recipes/info/<recipe_name>?format=toml`
///
/// NOTE this is accomplished this way because Rocket doesn't have a way to specify a
/// custom Content-Type for GET requests.
///
/// TODO Figure out how to add custom content types
#[get("/recipes/info/<recipe_name>?<format>", rank=3)]
pub fn recipes_info_toml(recipe_name: &str, format: Format, repo: State<RecipeRepo>) -> CORS<TOML<Recipe>> {
    info!("/recipes/info/ (TOML)"; "recipe_name" => recipe_name, "format" => format!("{:?}", format));
    // TODO Get the user's branch name. Use master for now.

    // TODO Error handling for format requests other than toml
    CORS(TOML(
        recipe::read(&repo.repo(), &recipe_name, "master", None).unwrap()
    ))
}

/// Hold the JSON response for /recipes/changes/
#[derive(Debug, Serialize)]
pub struct RecipesChangesResponse {
    recipes: Vec<RecipeCommitInfo>,
    offset:  i64,
    limit:   i64
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecipeCommitInfo {
    name:    String,
    changes: Vec<RecipeCommit>,
    total:   i64
}

/// Handler for `/recipes/changes/` with offset and limit arguments.
///
/// This calls [recipes_changes](fn.recipes_changes.html) with the optional `offset` and/or `limit`
/// values.
#[get("/recipes/changes/<recipes>?<filter>")]
pub fn recipes_changes_filter(recipes: &str, filter: Filter, repo: State<RecipeRepo>) -> CORS<JSON<RecipesChangesResponse>> {
    recipes_changes(recipes, filter.offset.unwrap_or(OFFSET), filter.limit.unwrap_or(LIMIT), repo)
}

/// Handler for `/recipes/changes/<recipes>`
///
/// This calls [recipes_changes](fn.recipes_changes.html) with the default `offset` and `limit` values.
#[get("/recipes/changes/<recipes>", rank=2)]
pub fn recipes_changes_default(recipes: &str, repo: State<RecipeRepo>) -> CORS<JSON<RecipesChangesResponse>> {
    recipes_changes(recipes, OFFSET, LIMIT, repo)
}

/// Return the changes to a recipe or list of recipes
///
/// # Arguments
///
/// * `offset` - Number of results to skip before returning results. Default is 0.
/// * `limit` - Maximum number of results to return. It may return less. Default is 20.
/// * `recipe_names` - Comma separated list of recipe names to return
///
/// # Response
///
/// * JSON response with recipe changes.
///
/// The changes for each listed recipe will have offset and limit applied to them.
/// This means that there will be cases where changes will be empty, when offset > total
/// for the recipe.
///
/// # Examples
///
/// ```json
/// {
///     "recipes": [
///         {
///             "name": "nfs-server",
///             "changes": [
///                 {
///                     "commit": "97d483e8dd0b178efca9a805e5fd8e722c48ac8e",
///                     "time": "Wed,  1 Mar 2017 13:29:37 -0800",
///                     "summary": "Recipe nfs-server saved"
///                 },
///                 {
///                     "commit": "857e1740f983bf033345c3242204af0ed7b81f37",
///                     "time": "Wed,  1 Mar 2017 09:28:53 -0800",
///                     "summary": "Recipe nfs-server saved"
///                 }
///             ],
///             "total": 2
///         },
///         {
///             "name": "ruby",
///             "changes": [
///                 {
///                     "commit": "4b84f072befc3f4debbe1348d6f4b166f7c83d78",
///                     "time": "Wed,  1 Mar 2017 13:32:09 -0800",
///                     "summary": "Recipe ruby saved"
///                 },
///                 {
///                     "commit": "85999253c1790367a860a344ea622971b7e0a050",
///                     "time": "Wed,  1 Mar 2017 13:31:19 -0800",
///                     "summary": "Recipe ruby saved"
///                 }
///             ],
///             "total": 2
///         }
///     ],
///     "offset": 0,
///     "limit": 20
/// }
/// ```
///
pub fn recipes_changes(recipe_names: &str, offset: i64, limit: i64, repo: State<RecipeRepo>) -> CORS<JSON<RecipesChangesResponse>> {
    info!("/recipes/changes/ (JSON)"; "recipe_names" => recipe_names, "offset" => offset, "limit" => limit);
    // TODO Get the user's branch name. Use master for now.

    let mut result = Vec::new();
    for name in recipe_names.split(",") {
        match recipe::commits(&repo.repo(), &name, "master") {
            Ok(mut commits) => {
                let total = commits.len() as i64;
                commits = commits.into_iter().skip(offset as usize).take(limit as usize).collect();
                result.push(RecipeCommitInfo {
                                name: name.to_string(),
                                changes: commits,
                                total: total
                });
            },
            Err(e) => {
                error!("Problem getting commits"; "recipe_name" => name, "error" => format!("{:?}", e));
            }
        }
    }
    result.sort();

    CORS(JSON(RecipesChangesResponse {
        recipes: result,
        offset:  offset,
        limit:   limit
    }))
}


/// Hold the JSON response for /recipes/diff/
#[derive(Debug, Serialize)]
pub struct RecipesDiffResponse {
    recipes: Vec<RecipeDiffInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecipeDiffInfo {
    name: String,
    from: String,
    to: String,
    diff: Vec<String>
}

/// Handler for `/recipes/diff/<recipe>/<from_commit>/<to_commit>`
///
/// Return the differences between two commits of a recipe
///
/// # Arguments
///
/// * `recipe_name` - Recipe name
/// * `from_commit` - The older commit to caclulate the difference from
/// * `to_commit` - The newer commit to calculate the diff. to or NEWEST
///
/// # Response
///
/// * JSON response with recipe changes.
///
///
/// # Examples
///
/// ```json
/// {
///     "recipes": [
///         {
///             "name": "nfs-server",
///             "from": "857e1740f983bf033345c3242204af0ed7b81f37",
///             "to": "NEWEST",
///             "diff": [
///                 "diff --git a/nfs-server.toml b/nfs-server.toml",
///                 "index 72b2953..adcf5e3 100644",
///                 "--- a/nfs-server.toml",
///                 "+++ b/nfs-server.toml",
///                 "@@ -5,3 +5,7 @@ name = \"nfs-server\"",
///                 " [[packages]]",
///                 " name = \"nfs\"",
///                 " version = \"4.1\"",
///                 "+",
///                 "+[[packages]]",
///                 "+name = \"NetworkManager\"",
///                 "+version = \"1.0.6\""
///             ]
///         }
///     ]
/// }
/// ```
///
#[get("/recipes/diff/<recipe_name>/<from_commit>/<to_commit>")]
pub fn recipes_diff(recipe_name: &str, from_commit: &str, to_commit: &str,
                    repo: State<RecipeRepo>) -> CORS<JSON<RecipesDiffResponse>> {
    info!("/recipes/diff/"; "recipe_name" => recipe_name,
                            "from_commit" => from_commit, "to_commit" => to_commit);
    // TODO Get the user's branch name. Use master for now.

    // Convert to_commit == NEWEST to None
    let new_commit = match to_commit {
        "NEWEST" => None,
        commit => Some(commit)
    };
    let diff = match recipe::diff(&repo.repo(), recipe_name, "master", from_commit, new_commit) {
        Ok(diff) => diff,
        Err(e) => {
            error!("Problem getting diff"; "recipe_name" => recipe_name, "error" => format!("{:?}", e));
            vec![]
        }
    };

    let result = RecipeDiffInfo {
        name: recipe_name.to_string(),
        from: from_commit.to_string(),
        to: to_commit.to_string(),
        diff: diff
    };

    CORS(JSON(RecipesDiffResponse {
        recipes: vec![result],
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
/// ## Recipe Version
///
/// The recipe version must be a valid [semver](http://www.semver.org) formatted version, blank, or missing.
/// If the version is valid, and matches the previously saved version, it will have the patch
/// number (z in x.y.z) incremented automatically.
///
/// If the version is missing or blank it will be set to "0.0.1"
///
/// If the new version doesn't match the last saved version, the new version will be used.
///
/// # Examples
///
/// ## POST body
///
/// ```json
/// {
///     "name": "http-server",
///     "description": "An example http server with PHP and MySQL support.",
///     "version": "0.0.1",
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
pub fn recipes_new_json(recipe: JSON<Recipe>, repo: State<RecipeRepo>) -> CORS<JSON<RecipesNewResponse>> {
    info!("/recipes/new/ (JSON)"; "recipe.name" => recipe.name);
    // TODO Get the user's branch name. Use master for now.

    let status = match recipe::write(&repo.repo(), &recipe, "master", None) {
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


/// Accept a TOML formatted POST to /recipes/new
///
/// This requires that the client set the type to "text/x-toml" and that the data be passed
/// without change.
///
/// eg. `curl -H "Content-Type: text/x-toml" -X POST --data-binary @nginx.toml http://API/URL`
///
#[post("/recipes/new", data="<recipe>", rank=2)]
pub fn recipes_new_toml(recipe: TOML<Recipe>, repo: State<RecipeRepo>) -> CORS<JSON<RecipesNewResponse>> {
    info!("/recipes/new/ (TOML)"; "recipe.name" => recipe.name);
    // TODO Get the user's branch name. Use master for now.

    let status = match recipe::write(&repo.repo(), &recipe, "master", None) {
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


/// Hold the JSON response for /recipes/new/
#[derive(Debug, Serialize)]
pub struct RecipesDeleteResponse {
    status: bool
}

/// Handler for `/recipes/delete/<recipe>`
/// Delete a recipe
///
/// # Arguments
///
/// * `recipe_name` - Recipe to delete
///
/// # Response
///
/// * JSON response with "status" set to true or false.
///
///
/// Only a DELETE request is valid. GET and POST are ignored.
///
/// ## Response
///
/// ```json
/// {
///     "status": true
/// }
/// ```
#[delete("/recipes/delete/<recipe_name>")]
pub fn recipes_delete(recipe_name: &str, repo: State<RecipeRepo>) -> CORS<JSON<RecipesDeleteResponse>> {
    info!("/recipes/delete/"; "recipe_name" => recipe_name);
    // TODO Get the user's branch name. Use master for now.

    let status = match recipe::delete(&repo.repo(), recipe_name, "master") {
        Ok(result) => result,
        Err(e) => {
            error!("recipes_delete"; "recipe_name" => recipe_name, "error" => format!("{:?}", e));
            false
        }
    };

    // TODO Return error information
    CORS(JSON(RecipesDeleteResponse {
            status: status
    }))
}


/// Hold the JSON response for /recipes/undo/
#[derive(Debug, Serialize)]
pub struct RecipesUndoResponse {
    status: bool
}

/// Handler for `/recipes/undo/<recipe>/<commit>`
/// Undo changes to a recipe by reverting to a previous commit
///
/// # Arguments
///
/// * `recipe_name` - Recipe to undo
/// * `commit` - Commit to revert to
///
/// # Response
///
/// * JSON response with "status" set to true or false.
///
///
/// Only a POST request is valid. GET is ignored.
///
/// ## Response
///
/// ```json
/// {
///     "status": true
/// }
/// ```
#[post("/recipes/undo/<recipe_name>/<commit>")]
pub fn recipes_undo(recipe_name: &str, commit: &str, repo: State<RecipeRepo>) -> CORS<JSON<RecipesUndoResponse>> {
    info!("/recipes/undo/"; "recipe_name" => recipe_name, "commit" => commit);
    // TODO Get the user's branch name. Use master for now.

    let status = match recipe::revert(&repo.repo(), recipe_name, "master", commit) {
        Ok(result) => result,
        Err(e) => {
            error!("recipes_undo"; "recipe_name" => recipe_name, "commit" => commit, "error" => format!("{:?}", e));
            false
        }
    };

    // TODO Return error information
    CORS(JSON(RecipesUndoResponse {
            status: status
    }))
}


/// A Recipe and its dependencies
#[derive(Debug, Serialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecipeDeps {
    recipe:       Recipe,
    modules:      Vec<PackageNEVRA>,
    dependencies: Vec<PackageNEVRA>
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
/// * JSON response like:
///   `{"recipes": [{"recipe": {RECIPE}, "modules": [NEVRA, ...], "dependencies": [NEVRA, ...]}]}`
///   Where RECIPE is the same JSON you would get from a /recipes/info/ query
///   NEVRA Is the name and version of a project build. modules are the versions chosen for the
///   modules and packages listed in the recipe. dependencies are all the dependencies needed to
///   satisfy the recipe.
///   Detailed info about the selected project can be requested with /modules/info/<name>
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
///                {
///                    "name": "httpd",
///                    "epoch": 0,
///                    "version": "2.4.6",
///                    "release": "40.el7",
///                    "arch": "x86_64"
///                },
///                {
///                    "name": "mod_auth_kerb",
///                    "epoch": 0,
///                    "version": "5.4",
///                    "release": "28.el7",
///                    "arch": "x86_64"
///                },
///                ...
///             ],
///             "dependencies": [
///                {
///                    "name": "acl",
///                    "epoch": 0,
///                    "version": "2.2.51",
///                    "release": "12.el7",
///                    "arch": "x86_64"
///                },
///                {
///                    "name": "apr",
///                    "epoch": 0,
///                    "version": "1.4.8",
///                    "release": "3.el7",
///                    "arch": "x86_64"
///                },
///                ...
///             ],
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
            let mut projects = Vec::new();
            projects.extend(recipe.clone().modules.iter().map(|m| m.name.clone()));
            projects.extend(recipe.clone().packages.iter().map(|p| p.name.clone()));
            projects.sort();
            projects.dedup();

            debug!("recipes_depsolve"; "projs" => format!("{:?}", projects));
            // deps for the whole recipe
            let pkg_nevras = depsolve_helper(&db.conn(), &projects);

            // Get the version chosen for each individual recipe module/package
            let mut recipe_nevras = Vec::new();
            for proj in projects {
                recipe_nevras.push(
                    match pkg_nevras.binary_search_by_key(&proj, |ref s| s.name.clone()) {
                        Ok(idx) => pkg_nevras[idx].clone(),
                        Err(_) => PackageNEVRA {
                            name:    proj,
                            epoch:   0,
                            version: "UNKNOWN".to_string(),
                            release: "".to_string(),
                            arch:    "".to_string()
                        }
                });
            }
            recipe_nevras.sort();

            result.push(RecipeDeps {
                recipe:       recipe,
                modules:      recipe_nevras,
                dependencies: pkg_nevras
            });
        });
    }
    result.sort();
    CORS(JSON(RecipesDepsolveResponse {
            recipes: result
    }))
}
