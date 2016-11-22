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
use std::path::Path;
use toml;

// bdcs database functions
use db::{get_builds_name, get_build_files, get_projects_name, get_project_kv_project_id, get_builds_project_id, get_build_kv_build_id, get_source_id, get_source_kv_source_id, };


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

// Recipe TOML Parsing
#[derive(Debug, RustcDecodable, RustcEncodable)]
struct RecipeList {
    name: Option<String>,
    description: Option<String>,
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
struct Recipe {
    name: Option<String>,
    description: Option<String>,
    modules: Option<Vec<Modules>>,
    packages: Option<Vec<Packages>>
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
struct Modules {
    name: Option<String>,
    version: Option<String>
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
struct Packages {
    name: Option<String>,
    version: Option<String>
}


// Composer API v0 Implementations
pub fn test_v0<'mw>(_req: &mut Request, res: Response<'mw>) -> MiddlewareResult<'mw> {
   res.send("API v0 test")
}

pub fn unimplemented_v0<'mw>(_req: &mut Request, res: Response<'mw>) -> MiddlewareResult<'mw> {
   res.error(StatusCode::ImATeapot, "API Not Yet Implemented.")
}

pub fn compose_types_v0<'mw>(_req: &mut Request, mut res: Response<'mw>) -> MiddlewareResult<'mw> {
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

    res.set(MediaType::Json);
    res.send(json::encode(&types).expect("Failed to serialize"))
}

pub fn dnf_info_packages_v0<'mw>(req: &mut Request, mut res: Response<'mw>) -> MiddlewareResult<'mw> {
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
pub fn project_list_v0<'mw>(req: &mut Request, mut res: Response<'mw>) -> MiddlewareResult<'mw> {
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
pub fn project_info_v0<'mw>(req: &mut Request, mut res: Response<'mw>) -> MiddlewareResult<'mw> {
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
/// [{"name": "name of recipe", "description": "description from recipe"}, ]
/// XXX I do not know how to pass in additional configuration data, like a recipe path
pub fn recipe_list_v0<'mw>(req: &mut Request, mut res: Response<'mw>) -> MiddlewareResult<'mw> {
    // This is more kludgy than normal because recipe_path_cfg should really come from main()
    let recipe_path_cfg = "/var/tmp/recipes/";
    // XXX Really? To add 1 character?
    let mut recipe_path: String = String::new();
    recipe_path = format!("{}*", recipe_path_cfg);

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
        recipe_list.push(recipe);
    }

    res.set(MediaType::Json);
    res.send(json::encode(&recipe_list).expect("Failed to serialize"))
}


pub fn get_recipe_v0<'mw>(req: &mut Request, mut res: Response<'mw>) -> MiddlewareResult<'mw> {
    // This is more kludgy than normal because recipe_path_cfg should really come from main()
    let recipe_path_cfg = "/var/tmp/recipes/";

    let offset: i64;
    let limit: i64;
    {
        let query = req.query();
        offset = query.get("offset").unwrap_or("").parse().unwrap_or(0);
        limit = query.get("limit").unwrap_or("").parse().unwrap_or(20);
    }
    let names = req.param("names").unwrap_or("").split(",");

    // XXX For now the filename matches the name. Later: Better retrieval
    let mut recipe_list = Vec::new();
    for name in names {
        // This is more kludgy than normal because recipe_path_cfg should really come from main()
        // XXX Really? To add 1 character?
        // TODO Needs to be sanitized!
        let mut recipe_path: String = String::new();
        recipe_path = format!("{}{}", recipe_path_cfg, name);

        for path in glob(&recipe_path).unwrap().filter_map(Result::ok) {
            // Parse the TOML recipe into a Recipe struct
            let mut input = String::new();
            let mut f = File::open(path).unwrap();
            f.read_to_string(&mut input).unwrap();
            let recipe: Recipe = toml::decode_str(&input).unwrap();
            recipe_list.push(recipe);
        }
    }

    res.set(MediaType::Json);
    res.send(json::encode(&recipe_list).expect("Failed to serialize"))
}


pub fn post_recipe_v0<'mw>(req: &mut Request, mut res: Response<'mw>) -> MiddlewareResult<'mw> {
    // This is more kludgy than normal because recipe_path_cfg should really come from main()
    let recipe_path_cfg = "/var/tmp/recipes/";

    // Parse the JSON into Recipe structs (XXX Why does this work here, and not below req.param?)
    let recipe = match req.json_as::<Recipe>() {
        Ok(recipe) => recipe,
        Err(err) => return res.error(StatusCode::InternalServerError, "Too many names.")
    };
    let recipe_toml = toml::encode::<Recipe>(&recipe);
    println!("{:?}", recipe_toml);

    let name = req.param("name").unwrap_or("");
    if name.find(',') != None {
        // TODO Need to define a common error response for bad API calls
        return res.error(StatusCode::InternalServerError, "Too many names.");
    }

    // TODO Needs to be sanitized!
    let mut recipe_path: String = String::new();
    recipe_path = format!("{}{}", recipe_path_cfg, name);
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
