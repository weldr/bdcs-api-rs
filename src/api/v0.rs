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
use rocket::http::Status;
use rocket::request::FromParam;
use rocket_contrib::JSON;
use rustc_serialize::json::{self, ToJson, Json};
use toml;

// bdcs database functions
use db::*;
use api::DB;

/// This is used for optional query parameters that filter the results
#[derive(FromForm)]
pub struct Filter {
    offset: Option<i64>,
    limit: Option<i64>
}

/// Recipe names
///
/// This is used to easily parse the recipe's TOML, keys that don't exist are ignored,
/// so this only parses the name of each recipe.
///
#[derive(Debug, RustcDecodable, RustcEncodable)]
struct RecipeList {
    name: Option<String>,
}

/// Composer Recipe
///
/// This is used to parse the full recipe's TOML, and to write a JSON representation of
/// the Recipe.
///
#[derive(Debug, RustcDecodable, RustcEncodable)]
struct Recipe {
    name: Option<String>,
    description: Option<String>,
    modules: Option<Vec<Modules>>,
    packages: Option<Vec<Packages>>
}

/// Recipe Modules
///
/// This is used for the Recipe's `modules` section and can be serialized
/// to/from JSON and TOML.
#[derive(Debug, RustcDecodable, RustcEncodable)]
struct Modules {
    name: Option<String>,
    version: Option<String>
}

/// Recipe Packages
///
/// This is used for the Recipe's `packages` section
#[derive(Debug, RustcDecodable, RustcEncodable)]
struct Packages {
    name: Option<String>,
    version: Option<String>
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

/// Return detailed information about a list of package names filtered by limit and/or offset
#[get("/projects/list?<filter>")]
pub fn projects_list_filter(filter: Filter, db: DB) -> JSON<ProjectsResponse> {
    projects_list(db, filter.offset.unwrap_or(0), filter.limit.unwrap_or(20))
}

// This catches the path when no query string was passed
#[get("/projects/list", rank=2)]
pub fn projects_list_default(db: DB) -> JSON<ProjectsResponse> {
    projects_list(db, 0, 20)
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
/// {"projects":[{"name":"389-ds-base","summary":"389 Directory Server (base)"},{"name":"ElectricFence","summary":"A debugger which detects memory allocation violations"},{"name":"GConf2","summary":"A process-transparent configuration system"},{"name":"GeoIP","summary":"Library for country/city/organization to IP address or hostname mapping"},{"name":"ImageMagick","summary":"An X application for displaying and manipulating images"},{"name":"LibRaw","summary":"Library for reading RAW files obtained from digital photo cameras"},{"name":"ModemManager","summary":"Mobile broadband modem management service"},{"name":"MySQL-python","summary":"An interface to MySQL"},{"name":"NetworkManager","summary":"Network connection manager and user applications"},{"name":"NetworkManager-libreswan","summary":"NetworkManager VPN plug-in for libreswan"},{"name":"ORBit2","summary":"A high-performance CORBA Object Request Broker"},{"name":"OpenEXR","summary":"OpenEXR runtime libraries"},{"name":"OpenIPMI","summary":"IPMI (Intelligent Platform Management Interface) library and tools"},{"name":"PackageKit","summary":"Package management service"},{"name":"PyGreSQL","summary":"A Python client library for PostgreSQL"},{"name":"PyPAM","summary":"PAM bindings for Python"},{"name":"PyQt4","summary":"Python bindings for Qt4"},{"name":"PyYAML","summary":"YAML parser and emitter for Python"},{"name":"Red_Hat_Enterprise_Linux-Release_Notes-7-as-IN","summary":"Assamese translation of Release_Notes"},{"name":"Red_Hat_Enterprise_Linux-Release_Notes-7-bn-IN","summary":"Bengali translation of Release_Notes"}]}
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
