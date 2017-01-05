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

/// This is used to hold the details about the availabe output types supported by composer
///
/// This will eventually come from a plugin system instead of being a static list constructed
/// by the handler.
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

/*
#[get("/test")]
fn test(db: DB) -> String {
    let recipe_path = config::active().unwrap().get_str("recipe_path").unwrap_or("./recipes");
    println!("{:?}", db.conn());

    let mut stmt = db.conn().prepare("
            select projects.*
            from projects
            where projects.name GLOB :project ORDER BY projects.id").unwrap();
    let mut rows = stmt.query_named(&[(":project", &"*")]).unwrap();

    let mut contents: Vec<String> = Vec::new();
    while let Some(row) = rows.next() {
        match row {
            Ok(row) => contents.push(row.get(1)),
            _ => ()
        }
    }

    format!("This is a test\nrecipe_path = {}\n{:?}\n", recipe_path, contents)
}

-    // Composer v0 API
-    server.get("/api/v0/isos", unimplemented_v0);
-    server.post("/api/v0/compose", unimplemented_v0);
-    server.get("/api/v0/compose/status", unimplemented_v0);
-    server.get("/api/v0/compose/status/:compose_id", unimplemented_v0);
-    server.get("/api/v0/compose/types", compose_types_v0);
-    server.get("/api/v0/compose/log/:kbytes", unimplemented_v0);
-    server.post("/api/v0/compose/cancel", unimplemented_v0);
-
-    server.get("/api/v0/dnf/transaction/:packages", unimplemented_v0);
-    server.get("/api/v0/dnf/info/:packages", dnf_info_packages_v0);
-
-    server.get("/api/v0/projects/list", project_list_v0);
-    server.get("/api/v0/projects/info/:projects", project_info_v0);
-
-    server.get("/api/v0/module/info/:modules", unimplemented_v0);
-    server.get("/api/v0/module/list", group_list_v0);
-    server.get("/api/v0/module/list/:groups", group_list_v0);
-
-    server.get("/api/v0/recipe/list", recipe_list_v0);
-    server.get("/api/v0/recipe/:names", get_recipe_v0);
-    server.post("/api/v0/recipe/:name", post_recipe_v0);

*/

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

#[get("compose/status")]
pub fn compose_status<'r>() -> &'static str {
    "Unimplemented"
}

#[get("compose/status/<id>")]
pub fn compose_status_id<'r>(id: &str) -> &'static str {
    "Unimplemented"
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
#[get("/compose/types", format="application/json")]
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
