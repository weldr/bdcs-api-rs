//! BDCS API v0
//!
//! Copyright (C) 2016
//! Red Hat, Inc.  All rights reserved.
//!
//! This program is free software; you can redistribute it and/or modify
//! it under the terms of the GNU General Public License as published by
//! the Free Software Foundation; either version 2 of the License, or
//! (at your option) any later version.
//!
//! This program is distributed in the hope that it will be useful,
//! but WITHOUT ANY WARRANTY; without even the implied warranty of
//! MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//! GNU General Public License for more details.
//!
//! You should have received a copy of the GNU General Public License
//! along with this program.  If not, see <http://www.gnu.org/licenses/>.
//!
use config::BDCSConfig;
use flate2::Compression;
use flate2::write::GzEncoder;
use glob::glob;
use hyper::header::{self, qitem};
use nickel::{MediaType, Request, Response, MiddlewareResult, QueryString, JsonBody};
use nickel::status::StatusCode;
use nickel_sqlite::SqliteRequestExtensions;
use rustc_serialize::json::{self, ToJson, Json};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::prelude::*;
use toml;

// bdcs database functions
use db::{get_builds_name, get_build_files, get_projects_name, get_project_kv_project_id, get_builds_project_id,
        get_build_kv_build_id, get_source_id, get_source_kv_source_id, get_groups_name};


#[derive(RustcEncodable)]
struct ComposeTypes {
    name: String,
    enabled: bool
}

impl ComposeTypes {
    fn new<S: Into<String>>(name: S, enabled: bool) -> ComposeTypes {
        ComposeTypes { name: name.into(), enabled: enabled }
    }
}

impl ToJson for ComposeTypes {
    fn to_json(&self) -> Json {
        let mut d = BTreeMap::new();
        d.insert("name".to_string(), self.name.to_json());
        d.insert("enabled".to_string(), self.enabled.to_json());
        Json::Object(d)
    }
}


// Recipe TOML Parsing
#[derive(Debug, RustcDecodable, RustcEncodable)]
struct RecipeList {
    name: Option<String>,
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
struct Recipe {
    name: Option<String>,
    description: Option<String>,
    modules: Option<Vec<Modules>>,
    packages: Option<Vec<Packages>>
}

impl ToJson for Recipe {
    fn to_json(&self) -> Json {
        let mut d = BTreeMap::new();
        d.insert("name".to_string(), self.name.to_json());
        d.insert("description".to_string(), self.description.to_json());
        d.insert("modules".to_string(), self.modules.to_json());
        d.insert("packages".to_string(), self.packages.to_json());
        Json::Object(d)
    }
}


#[derive(Debug, RustcDecodable, RustcEncodable)]
struct Modules {
    name: Option<String>,
    version: Option<String>
}

impl ToJson for Modules {
    fn to_json(&self) -> Json {
        let mut d = BTreeMap::new();
        d.insert("name".to_string(), self.name.to_json());
        d.insert("version".to_string(), self.version.to_json());
        Json::Object(d)
    }
}


#[derive(Debug, RustcDecodable, RustcEncodable)]
struct Packages {
    name: Option<String>,
    version: Option<String>
}

impl ToJson for Packages {
    fn to_json(&self) -> Json {
        let mut d = BTreeMap::new();
        d.insert("name".to_string(), self.name.to_json());
        d.insert("version".to_string(), self.version.to_json());
        Json::Object(d)
    }
}


// Composer API v0 Implementations
pub fn test_v0<'mw>(_req: &mut Request<BDCSConfig>, res: Response<'mw, BDCSConfig>) -> MiddlewareResult<'mw, BDCSConfig> {
   res.send("API v0 test")
}

pub fn unimplemented_v0<'mw>(_req: &mut Request<BDCSConfig>, res: Response<'mw, BDCSConfig>) -> MiddlewareResult<'mw, BDCSConfig> {
   res.error(StatusCode::ImATeapot, "API Not Yet Implemented.")
}

pub fn compose_types_v0<'mw>(_req: &mut Request<BDCSConfig>, mut res: Response<'mw, BDCSConfig>) -> MiddlewareResult<'mw, BDCSConfig> {
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

    let mut response: BTreeMap<String, json::Json> = BTreeMap::new();
    response.insert("types".to_string(), types.to_json());

    res.set(MediaType::Json);
    res.send(json::encode(&response).expect("Failed to serialize"))
}

pub fn dnf_info_packages_v0<'mw>(req: &mut Request<BDCSConfig>, res: Response<'mw, BDCSConfig>) -> MiddlewareResult<'mw, BDCSConfig> {
    // Get the build details for NM
    let packages = req.param("packages").unwrap_or("").split(",");

    // Why does passing 'foo' match the route and passing: 'foo.1.1'
    // fail?

    let conn = req.db_conn().expect("Failed to get a database connection from the pool.");
    for pkg in packages {
        let result = get_builds_name(&conn, pkg);
        match result {
            Ok(builds) => {
                println!("===> package = {}", pkg);
                for build in builds {
                    println!("{:?}", build);
                    let s = String::from_utf8(build.changelog);
                    println!("Changelog:\n{}", s.unwrap());
                    println!("Files for build:");
                    let file_results = get_build_files(&conn, build.id);
                    match file_results {
                        Ok(files) => {
                            for f in files {
                                println!("{:?}", f);
                            }
                        }
                        Err(err) => println!("Error: {}", err)
                    }
                }
            }
            Err(err) => println!("Error: {}", err)
        }
    }
//    res.set(MediaType::Json);
    res.send("Write This")
}

/// List all of the available projects
pub fn project_list_v0<'mw>(req: &mut Request<BDCSConfig>, mut res: Response<'mw, BDCSConfig>) -> MiddlewareResult<'mw, BDCSConfig> {
    let offset: i64;
    let limit: i64;
    {
        let query = req.query();
        offset = query.get("offset").unwrap_or("").parse().unwrap_or(0);
        limit = query.get("limit").unwrap_or("").parse().unwrap_or(20);
    }

    let conn = req.db_conn().expect("Failed to get a database connection from the pool.");
    let mut project_list = Vec::new();
    let result = get_projects_name(&conn, "*", offset, limit);
    match result {
        Ok(projs) => {
            // SQL query could potentially return more than one, so loop.
            for p in projs {
                let mut proj_map: BTreeMap<String, json::Json> = BTreeMap::new();
                proj_map.insert("name".to_string(), p.name.to_json());
                proj_map.insert("summary".to_string(), p.summary.to_json());
                project_list.push(proj_map);
            }
        }
        Err(err) => println!("Error: {}", err)
    }

    res.set(MediaType::Json);


    // TODO Make this some kind of middleware thing
    match req.origin.headers.get::<header::AcceptEncoding>() {
        Some(header) => {
            if header.contains(&qitem(header::Encoding::Gzip)) {
                // Client accepts gzip, go ahead and compress it
                res.set(header::ContentEncoding(vec![header::Encoding::Gzip]));

                let mut encoder = GzEncoder::new(Vec::new(), Compression::Default);
                let _ = encoder.write(json::encode(&project_list).expect("Failed to serialize").as_bytes());
                return res.send(encoder.finish().unwrap());
            }
        }
        None => ()
    }
    res.send(json::encode(&project_list).expect("Failed to serialize"))
}

 /// Get information about a project
pub fn project_info_v0<'mw>(req: &mut Request<BDCSConfig>, mut res: Response<'mw, BDCSConfig>) -> MiddlewareResult<'mw, BDCSConfig> {
    let offset: i64;
    let limit: i64;
    {
        let query = req.query();
        offset = query.get("offset").unwrap_or("").parse().unwrap_or(0);
        limit = query.get("limit").unwrap_or("").parse().unwrap_or(20);
    }
    let projects = req.param("projects").unwrap_or("").split(",");

    // Why does passing 'foo' match the route and passing: 'foo.1.1'
    // fail?

    let conn = req.db_conn().expect("Failed to get a database connection from the pool.");
    let mut project_info = Vec::new();
    for proj in projects {
        let result = get_projects_name(&conn, proj, offset, limit);
        match result {
            Ok(projs) => {
                // SQL query could potentially return more than one, so loop.
                for p in projs {
                    let mut proj_map: BTreeMap<String, json::Json> = BTreeMap::new();
                    proj_map.insert("name".to_string(), p.name.to_json());
                    proj_map.insert("summary".to_string(), p.summary.to_json());
                    proj_map.insert("description".to_string(), p.description.to_json());
                    proj_map.insert("homepage".to_string(), p.homepage.unwrap_or("".to_string()).to_json());
                    proj_map.insert("upstream_vcs".to_string(), p.upstream_vcs.to_json());

                    // Add the project's key:value mappings
                    let result_2 = get_project_kv_project_id(&conn, p.id);
                    match result_2 {
                        Ok(kvs) => {
                            for kv in kvs {
                                proj_map.entry(kv.key_value.to_string()).or_insert(kv.val_value.to_json());
                            }
                        }
                        Err(err) => println!("Error: {}", err)
                    }


                    let mut builds_list = Vec::new();
                    let result_3 = get_builds_project_id(&conn, p.id);
                    match result_3 {
                        Ok(builds) => {
                            for b in builds {
                                let mut build_map: BTreeMap<String, json::Json> = BTreeMap::new();
                                build_map.insert("epoch".to_string(), b.epoch.to_json());
                                build_map.insert("release".to_string(), b.release.to_json());
                                build_map.insert("arch".to_string(), b.arch.to_json());
                                build_map.insert("build_time".to_string(), b.build_time.to_json());

                                // changelog is a Vec[u8] so convert it to a String
                                let s = String::from_utf8(b.changelog).unwrap_or("".to_string());
                                build_map.insert("changelog".to_string(), s.to_json());

                                build_map.insert("build_config_ref".to_string(), b.build_config_ref.to_json());
                                build_map.insert("build_env_ref".to_string(), b.build_env_ref.to_json());

                                let result_4 = get_build_kv_build_id(&conn, b.id);
                                match result_4 {
                                    Ok(kvs) => {
                                        for kv in kvs {
                                            build_map.entry(kv.key_value.to_string()).or_insert(kv.val_value.to_json());
                                        }
                                    }
                                    Err(err) => println!("Error: {}", err)
                                }

                                let result_5 = get_source_id(&conn, b.source_id);
                                match result_5 {
                                    // FIXME Only one possible result, not a Vec
                                    Ok(sources) => {
                                        for s in sources {
                                            build_map.insert("license".to_string(), s.license.to_json());
                                            build_map.insert("version".to_string(), s.version.to_json());
                                            build_map.insert("source_ref".to_string(), s.source_ref.to_json());
                                        }
                                    }
                                    Err(err) => println!("Error: {}", err)
                                }

                                let result_6 = get_source_kv_source_id(&conn, b.source_id);
                                match result_6 {
                                    Ok(kvs) => {
                                        for kv in kvs {
                                            build_map.entry(kv.key_value.to_string()).or_insert(kv.val_value.to_json());
                                        }
                                    }
                                    Err(err) => println!("Error: {}", err)
                                }


                                builds_list.push(Json::Object(build_map));
                            }
                        }
                        Err(err) => println!("Error: {}", err)
                    }
                    proj_map.insert("builds".to_string(), builds_list.to_json());
                    project_info.push(proj_map);
                }
            }
            Err(err) => println!("Error: {}", err)
        }
    }

    res.set(MediaType::Json);
    res.send(json::encode(&project_info).expect("Failed to serialize"))
}

/// Fetch the list of available recipes
/// { "recipes": ["name1", "name2", ...] }
pub fn recipe_list_v0<'mw>(req: &mut Request<BDCSConfig>, mut res: Response<'mw, BDCSConfig>) -> MiddlewareResult<'mw, BDCSConfig> {
    let bdcs_config = req.server_data();
    let recipe_path = bdcs_config.recipe_path.to_string() + "*";

    let offset: i64;
    let limit: i64;
    {
        let query = req.query();
        offset = query.get("offset").unwrap_or("").parse().unwrap_or(0);
        limit = query.get("limit").unwrap_or("").parse().unwrap_or(20);
    }

    let mut recipe_list = Vec::new();
    for path in glob(&recipe_path).unwrap().filter_map(Result::ok) {
        // Parse the TOML recipe into a Recipe struct
        let mut input = String::new();
        let mut f = File::open(path).unwrap();
        f.read_to_string(&mut input).unwrap();
        let recipe: RecipeList = toml::decode_str(&input).unwrap();
        recipe_list.push(recipe.name);
    }

    let mut response: BTreeMap<String, json::Json> = BTreeMap::new();
    response.insert("recipes".to_string(), recipe_list.to_json());

    res.set(MediaType::Json);
    res.send(json::encode(&response).expect("Failed to serialize"))
}


/// Return the contents of a recipe or list of recipes as JSON
/// { "name1": { "name": "name1", ... }, ... }
pub fn get_recipe_v0<'mw>(req: &mut Request<BDCSConfig>, mut res: Response<'mw, BDCSConfig>) -> MiddlewareResult<'mw, BDCSConfig> {
    let bdcs_config = req.server_data();

    let offset: i64;
    let limit: i64;
    {
        let query = req.query();
        offset = query.get("offset").unwrap_or("").parse().unwrap_or(0);
        limit = query.get("limit").unwrap_or("").parse().unwrap_or(20);
    }
    let names = req.param("names").unwrap_or("").split(",");

    // XXX For now the filename matches the name. Later: Better retrieval
    let mut response: BTreeMap<String, json::Json> = BTreeMap::new();
    for name in names {
        // TODO Needs to be sanitized!
        let recipe_path = bdcs_config.recipe_path.to_string() + name;

        for path in glob(&recipe_path).unwrap().filter_map(Result::ok) {
            // Parse the TOML recipe into a Recipe struct
            let mut input = String::new();
            let mut file = match File::open(&path) {
                Ok(file) => file,
                Err(err) => {
                    println!("Error reading {:?}: {}", path, err);
                    return res.error(StatusCode::InternalServerError, "File Open Error.")
                }
            };
            match file.read_to_string(&mut input) {
                Ok(_) => println!("Read recipe from {:?}", path),
                Err(err) => {
                    println!("Error reading {:?}: {}", path, err);
                    return res.error(StatusCode::InternalServerError, "Read Error.")
                }
            };
            let recipe = match toml::decode_str::<Recipe>(&input) {
                Some(recipe) => recipe,
                None => return res.error(StatusCode::InternalServerError, "Error parsing TOML")
            };

            // XXX Is this the right way to do this?
            let name = recipe.name.as_ref().unwrap();
            response.insert(name.to_string(), recipe.to_json());
        }
    }

    res.set(MediaType::Json);
    res.send(json::encode(&response).expect("Failed to serialize"))
}


pub fn post_recipe_v0<'mw>(req: &mut Request<BDCSConfig>, mut res: Response<'mw, BDCSConfig>) -> MiddlewareResult<'mw, BDCSConfig> {
    let bdcs_config = req.server_data();

    // Parse the JSON into Recipe structs (XXX Why does this work here, and not below req.param?)
    let recipe = match req.json_as::<Recipe>() {
        Ok(recipe) => recipe,
        Err(err) => {
            println!("Error parsing JSON: {}", err);
            return res.error(StatusCode::InternalServerError, "Error parsing JSON")
        }
    };
    let recipe_toml = toml::encode::<Recipe>(&recipe);
    println!("{:?}", recipe_toml);

    let name = req.param("name").unwrap_or("");
    if name.find(',') != None {
        // TODO Need to define a common error response for bad API calls
        return res.error(StatusCode::InternalServerError, "Too many names.");
    }

    // TODO Needs to be sanitized!
    let recipe_path = bdcs_config.recipe_path.to_string() + name;
    let mut file = match File::create(&recipe_path) {
        Ok(file) => file,
        Err(err) => {
            println!("Error opening {} for write: {}", recipe_path, err);
            return res.error(StatusCode::InternalServerError, "Error opening file.")
        }
    };
    match file.write_all(toml::encode_str(&recipe_toml).as_bytes()) {
        Ok(_) => println!("Wrote Recipe to {}", recipe_path),
        Err(err) => {
            println!("Error writing {}: {}", recipe_path, err);
            return res.error(StatusCode::InternalServerError, "Error writing file.")
        }
    };

    res.set(StatusCode::Ok);
    res.send("")
}

/// List the available groups
/// AKA modules
/// { "modules": [{"name": "group1", ...}, ...] }
pub fn group_list_v0<'mw>(req: &mut Request<BDCSConfig>, mut res: Response<'mw, BDCSConfig>) -> MiddlewareResult<'mw, BDCSConfig> {
    let offset: i64;
    let limit: i64;
    {
        let query = req.query();
        offset = query.get("offset").unwrap_or("").parse().unwrap_or(0);
        limit = query.get("limit").unwrap_or("").parse().unwrap_or(20);
    }

    // List all groups if there is no groups param or if it is empty.
    let groups = match req.param("groups") {
        Some(groups) => if groups.len() > 0 { groups } else {"*"},
        None => "*"
    };

    let conn = req.db_conn().expect("Failed to get a database connection from the pool.");
    let mut group_list = Vec::new();
    for group in groups.split(",") {
        let result = get_groups_name(&conn, group, offset, limit);
        match result {
            Ok(grps) => {
                // SQL query could potentially return more than one, so loop.
                for g in grps {
                    let mut group_map: BTreeMap<String, json::Json> = BTreeMap::new();
                    group_map.insert("name".to_string(), g.name.to_json());
                    group_map.insert("group_type".to_string(), g.group_type.to_json());
                    group_list.push(group_map);
                }
            }
            Err(err) => println!("Error: {}", err)
        }
    }
    res.set(MediaType::Json);

    let mut response: BTreeMap<String, json::Json> = BTreeMap::new();
    response.insert("modules".to_string(), group_list.to_json());

    // TODO Make this some kind of middleware thing
    match req.origin.headers.get::<header::AcceptEncoding>() {
        Some(header) => {
            if header.contains(&qitem(header::Encoding::Gzip)) {
                // Client accepts gzip, go ahead and compress it
                res.set(header::ContentEncoding(vec![header::Encoding::Gzip]));

                let mut encoder = GzEncoder::new(Vec::new(), Compression::Default);
                let _ = encoder.write(json::encode(&response).expect("Failed to serialize").as_bytes());
                return res.send(encoder.finish().unwrap());
            }
        }
        None => ()
    }
    res.send(json::encode(&response).expect("Failed to serialize"))
}
