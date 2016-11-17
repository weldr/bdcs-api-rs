//! BDCS API
//!
//! <Copyright>
//!
//! Note: This requires sqlite-devel, and openssl-devel on the host in order to build

#[macro_use] extern crate nickel;
extern crate hyper;
extern crate rustc_serialize;
extern crate rusqlite;
extern crate nickel_sqlite;
extern crate r2d2;
extern crate r2d2_sqlite;
extern crate unicase;
extern crate getopts;
extern crate flate2;

use std::env;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::collections::BTreeMap;
use std::io::Write;

// Database
use rusqlite::Connection;
use getopts::Options;
use r2d2::{Pool, Config};
use r2d2_sqlite::SqliteConnectionManager;

// API Framework
use nickel::{Nickel, MediaType, HttpRouter, Request, Response, MiddlewareResult, NickelError};
use nickel::status::StatusCode;
use nickel_sqlite::{SqliteMiddleware, SqliteRequestExtensions};
use hyper::header::{self, qitem};
use unicase::UniCase;
use rustc_serialize::json::{self, ToJson, Json};
use flate2::Compression;
use flate2::write::GzEncoder;


fn print_usage(program: &str, opts: Options) {
    println!("{}", opts.usage(&format!("Usage: {} [options] <sqlite.db>", program)));
}

/// bdcs database schema structs
#[derive(Debug)]
struct Projects {
    id: i64,
    name: String,
    summary: String,
    description: String,
    homepage: Option<String>,
    upstream_vcs: String
}

#[derive(Debug)]
struct Sources {
    id: i64,
    project_id: i64,
    license: String,
    version: String,
    source_ref: String
}

#[derive(Debug)]
struct Builds {
    id: i64,
    source_id: i64,
    epoch: i64,
    release: String,
    arch: String,
    build_time: String,         // Should be Timespec or something like that
    changelog: Vec<u8>,
    build_config_ref: String,
    build_env_ref: String,
}

#[derive(Debug)]
struct BuildSignatures {
    id: i64,
    build_id: i64,
    signature_type: String,
    signature_data: Vec<u8>
}

#[derive(Debug)]
struct Files {
    id: i64,
    path: String,           // Could use rust's Path type?
    digest: String,
    file_type: String,
    file_mode: i64,
    file_user: String,
    file_group: String,
    file_size: i64,
    mtime: i64,
    symlink_target: Option<String>,
}

#[derive(Debug)]
enum FileAttrValues {
    file_id,
    attribute_type,
    attribute_value
}

#[derive(Debug)]
struct FileAttributes {
    id: i64,
    file_id: i64,
    attribute_type: String,
    attribute_value: String,
    FileIdKey: i64,
    TypeKey: FileAttrValues,
    XattrKey: FileAttrValues
}

#[derive(Debug)]
struct BuildFiles {
    id: i64,
    build_id: i64,
    file_id: i64
}

#[derive(Debug)]
struct KeyVal {
    id: i64,
    key_value: String,
    val_value: String
}

#[derive(Debug)]
struct ProjectKeyValues {
    id: i64,
    project_id: i64,
    key_val_id: i64
}

#[derive(Debug)]
struct SourceKeyValues {
    id: i64,
    source_id: i64,
    key_val_id: i64
}

#[derive(Debug)]
struct BuildKeyValues {
    id: i64,
    build_id: i64,
    key_val_id: i64
}

#[derive(Debug)]
struct FileKeyValues {
    id: i64,
    file_id: i64,
    key_val_id: i64
}

/// List contents of a package given by name.
fn get_pkg_files_name(conn: &Connection, pkgname: &str) -> rusqlite::Result<Vec<PathBuf>> {
    let mut stmt = try!(conn.prepare("
            select files.path
            from files, key_val, file_key_values
            on key_val.id == file_key_values.key_val_id and
               file_key_values.file_id == files.id
            where key_val.key_value == 'packageName' and
                  key_val.val_value == :pkgname"));
    let mut rows = try!(stmt.query_named(&[(":pkgname", &pkgname)]));

    let mut contents = Vec::new();
    while let Some(row) = rows.next() {
        let path: String = try!(row).get(0);
        contents.push(PathBuf::from(path));
    }
    Ok(contents)
}

// Use a package struct to describe the package?
// How to make these queries easier to expose as a library?

/// List contents of a package given by NEVRA.
fn get_pkg_files_nevra (conn: &Connection, pkgname: &str,
                                           epoch: i64,
                                           version: &str,
                                           release: &str,
                                           arch: &str) -> rusqlite::Result<Vec<PathBuf>> {
    let mut stmt = try!(conn.prepare("
            select files.path
            from projects, sources, builds, files, build_files, key_val, file_key_values
            on key_val.id == file_key_values.key_val_id and
               file_key_values.file_id == files.id and
               sources.project_id == projects.id and
               builds.source_id == sources.id and
               build_files.build_id == builds.id and
               build_files.file_id == files.id
            where key_val.key_value == 'packageName' and
                  key_val.val_value == :pkgname and
                  sources.version == :version and
                  builds.epoch == :epoch and
                  builds.release == :release and
                  builds.arch == :arch"));
    let mut rows = try!(stmt.query_named(&[(":pkgname", &pkgname),
                                           (":epoch", &epoch),
                                           (":version", &version),
                                           (":release", &release),
                                           (":arch", &arch)]));

    let mut contents = Vec::new();
    while let Some(row) = rows.next() {
        let path: String = try!(row).get(0);
        contents.push(PathBuf::from(path));
    }
    Ok(contents)
}


/// Find all builds that match a given project name.
fn get_builds_name(conn: &Connection, project: &str) -> rusqlite::Result<Vec<Builds>> {
    let mut stmt = try!(conn.prepare("
            select builds.*
            from builds, sources, projects
            on builds.source_id == sources.id and
               sources.project_id == projects.id
            where projects.name == :project"));
    let mut rows = try!(stmt.query_named(&[(":project", &project)]));

    let mut contents = Vec::new();
    while let Some(row) = rows.next() {
        let row = try!(row);
        // Sure would be nice not to use indexes here!
        contents.push(Builds {
                        id: row.get(0),
                        source_id: row.get(1),
                        epoch: row.get(2),
                        release: row.get(3),
                        arch: row.get(4),
                        build_time: row.get(5),
                        changelog: row.get(6),
                        build_config_ref: row.get(7),
                        build_env_ref: row.get(8),
                    });
        // NOTE: build_time should be some kind of time type, but crashed with Timespec because the
        // format used is incompatible (I think it is the T instead of ' ' in the middle)
        // changelog is a BLOB which is a Vec[u8] so it needs to be converted to a String with
        // .from _utf8() to be useful.
    }
    Ok(contents)
}

/// List contents of a build.
fn get_build_files(conn: &Connection, build_id: i64) -> rusqlite::Result<Vec<PathBuf>> {
    let mut stmt = try!(conn.prepare("
            select files.path
            from files, build_files, builds
            on files.id == build_files.file_id and
               builds.id == build_files.build_id
            where builds.id == :build_id"));
    let mut rows = try!(stmt.query_named(&[(":build_id", &build_id)]));

    let mut contents = Vec::new();
    while let Some(row) = rows.next() {
        let path: String = try!(row).get(0);
        contents.push(PathBuf::from(path));
    }
    Ok(contents)
}

/// List all builds containing a filename path
fn get_builds_filename(conn: &Connection, filename: &str) -> rusqlite::Result<Vec<Builds>> {
    let mut stmt = try!(conn.prepare("
            select builds.*
            from builds, files, build_files
            on build_files.build_id == builds.id and
               build_files.file_id == files.id
            where files.path == :filename"));
    let mut rows = try!(stmt.query_named(&[(":filename", &filename)]));

    let mut contents = Vec::new();
    while let Some(row) = rows.next() {
        let row = try!(row);
        // Sure would be nice not to use indexes here!
        contents.push(Builds {
                        id: row.get(0),
                        source_id: row.get(1),
                        epoch: row.get(2),
                        release: row.get(3),
                        arch: row.get(4),
                        build_time: row.get(5),
                        changelog: row.get(6),
                        build_config_ref: row.get(7),
                        build_env_ref: row.get(8),
                    });
        // NOTE: build_time should be some kind of time type, but crashed with Timespec because the
        // format used is incompatible (I think it is the T instead of ' ' in the middle)
        // changelog is a BLOB which is a Vec[u8] so it needs to be converted to a String with
        // .from _utf8() to be useful.
    }
    Ok(contents)
}

/// Find all projects that contain a given filename.
fn get_projects_filename(conn: &Connection, filename: &str) -> rusqlite::Result<Vec<Projects>> {
    let mut stmt = try!(conn.prepare("
            select projects.*
            from builds, files, build_files, sources, projects
            on builds.id == build_files.build_id and
               files.id == build_files.file_id and
               builds.source_id == sources.id and
               sources.project_id == projects.id
            where files.path == :filename"));
    let mut rows = try!(stmt.query_named(&[(":filename", &filename)]));

    let mut contents = Vec::new();
    while let Some(row) = rows.next() {
        let row = try!(row);
        // Sure would be nice not to use indexes here!
        contents.push(Projects {
                        id: row.get(0),
                        name: row.get(1),
                        summary: row.get(2),
                        description: row.get(3),
                        homepage: row.get(4),
                        upstream_vcs: row.get(5)
                    });
    }
    Ok(contents)
}

/// Find all projects matching a name
fn get_projects_name(conn: &Connection, project: &str) -> rusqlite::Result<Vec<Projects>> {
    let mut stmt = try!(conn.prepare("
            select projects.*
            from projects
            where projects.name GLOB :project"));
    let mut rows = try!(stmt.query_named(&[(":project", &project)]));

    let mut contents = Vec::new();
    while let Some(row) = rows.next() {
        let row = try!(row);
        // Sure would be nice not to use indexes here!
        contents.push(Projects {
                        id: row.get(0),
                        name: row.get(1),
                        summary: row.get(2),
                        description: row.get(3),
                        homepage: row.get(4),
                        upstream_vcs: row.get(5)
                    });
    }
    Ok(contents)
}

/// Find all sources matching a source id
fn get_source_id(conn: &Connection, source_id: i64) -> rusqlite::Result<Vec<Sources>> {
    let mut stmt = try!(conn.prepare("
            select sources.*
            from sources
            where sources.id == :source_id"));
    let mut rows = try!(stmt.query_named(&[(":source_id", &source_id)]));

    let mut contents = Vec::new();
    while let Some(row) = rows.next() {
        let row = try!(row);
        // Sure would be nice not to use indexes here!
        contents.push(Sources {
                        id: row.get(0),
                        project_id: row.get(1),
                        license: row.get(2),
                        version: row.get(3),
                        source_ref: row.get(4)
                 });
    }
    Ok(contents)
}

/// Get builds for a project based on project id
fn get_builds_project_id(conn: &Connection, project_id: i64) -> rusqlite::Result<Vec<Builds>> {
    let mut stmt = try!(conn.prepare("
            select builds.*
            from builds, sources, projects
            on builds.source_id == sources.id and
               sources.project_id == projects.id
            where projects.id == :project_id"));
    let mut rows = try!(stmt.query_named(&[(":project_id", &project_id)]));

    let mut contents = Vec::new();
    while let Some(row) = rows.next() {
        let row = try!(row);
        // Sure would be nice not to use indexes here!
        contents.push(Builds {
                        id: row.get(0),
                        source_id: row.get(1),
                        epoch: row.get(2),
                        release: row.get(3),
                        arch: row.get(4),
                        build_time: row.get(5),
                        changelog: row.get(6),
                        build_config_ref: row.get(7),
                        build_env_ref: row.get(8),
                    });
        // NOTE: build_time should be some kind of time type, but crashed with Timespec because the
        // format used is incompatible (I think it is the T instead of ' ' in the middle)
        // changelog is a BLOB which is a Vec[u8] so it needs to be converted to a String with
        // .from _utf8() to be useful.
    }
    Ok(contents)
}


/// Get k:v data for project based on project id
fn get_project_kv_project_id(conn: &Connection, project_id: i64) -> rusqlite::Result<Vec<KeyVal>> {
    let mut stmt = try!(conn.prepare("
            select key_val.*
            from project_key_values, key_val
            on key_val.id == project_key_values.key_val_id
            where project_key_values.package_id == :project_id"));
    let mut rows = try!(stmt.query_named(&[(":project_id", &project_id)]));

    let mut contents = Vec::new();
    while let Some(row) = rows.next() {
        let row = try!(row);
        // Sure would be nice not to use indexes here!
        contents.push(KeyVal {
                        id: row.get(0),
                        key_value: row.get(1),
                        val_value: row.get(2),
                    });
    }
    Ok(contents)
}

/// Get k:v data for sources based on id
fn get_source_kv_source_id(conn: &Connection, source_id: i64) -> rusqlite::Result<Vec<KeyVal>> {
    let mut stmt = try!(conn.prepare("
            select key_val.*
            from source_key_values, key_val
            on key_val.id == source_key_values.key_val_id
            where source_key_values.source_id == :source_id"));
    let mut rows = try!(stmt.query_named(&[(":source_id", &source_id)]));

    let mut contents = Vec::new();
    while let Some(row) = rows.next() {
        let row = try!(row);
        // Sure would be nice not to use indexes here!
        contents.push(KeyVal {
                        id: row.get(0),
                        key_value: row.get(1),
                        val_value: row.get(2),
                    });
    }
    Ok(contents)
}


/// Get k:v data for builds based on id
fn get_build_kv_build_id(conn: &Connection, build_id: i64) -> rusqlite::Result<Vec<KeyVal>> {
    let mut stmt = try!(conn.prepare("
            select key_val.*
            from build_key_values, key_val
            on key_val.id == build_key_values.key_val_id
            where build_key_values.build_id == :build_id"));
    let mut rows = try!(stmt.query_named(&[(":build_id", &build_id)]));

    let mut contents = Vec::new();
    while let Some(row) = rows.next() {
        let row = try!(row);
        // Sure would be nice not to use indexes here!
        contents.push(KeyVal {
                        id: row.get(0),
                        key_value: row.get(1),
                        val_value: row.get(2),
                    });
    }
    Ok(contents)
}


fn bcl_queries(conn: &Connection) {
    // Run some queries
//    let result = get_pkg_files_name(&conn, "lorax");
    let result = get_pkg_files_nevra(&conn, "lorax", 0, "19.6.78", "1.el7", "x86_64");
    match result {
        Ok(files) => {
            for f in files {
                println!("{:?}", f);
            }
        }
        Err(err) => println!("Error: {}", err)
    }

    // Get the builds that include /usr/bin/ls
    let result = get_builds_filename(&conn, "/usr/bin/ls");
    match result {
        Ok(builds) => {
            for build in builds {
                println!("{:?}", build);
                let s = String::from_utf8(build.changelog);
                println!("Changelog:\n{}", s.unwrap());
            }
        }
        Err(err) => println!("Error: {}", err)
    }

    // Get the projects that include /usr/bin/ls
    let result = get_projects_filename(&conn, "/usr/bin/ls");
    match result {
        Ok(projects) => {
            for project in projects {
                println!("{:?}", project);
            }
        }
        Err(err) => println!("Error: {}", err)
    }
}


// Composer API v0 Implementations
fn test_v0<'mw>(_req: &mut Request, res: Response<'mw>) -> MiddlewareResult<'mw> {
   res.send("API v0 test")
}

fn unimplemented_v0<'mw>(_req: &mut Request, res: Response<'mw>) -> MiddlewareResult<'mw> {
   res.error(StatusCode::ImATeapot, "API Not Yet Implemented.")
}

fn compose_types_v0<'mw>(_req: &mut Request, mut res: Response<'mw>) -> MiddlewareResult<'mw> {
    let mut types = HashMap::new();
    types.insert("iso", true);
    types.insert("disk-image", false);
    types.insert("fs-image", false);
    types.insert("ami", false);
    types.insert("tar", false);
    types.insert("live-pxe", false);
    types.insert("live-ostree", false);
    types.insert("oci", false);
    types.insert("vagrant", false);
    types.insert("qcow2", false);
    types.insert("vmdk", false);
    types.insert("vhdx", false );

    res.set(MediaType::Json);
    res.send(json::encode(&types).expect("Failed to serialize"))
}

fn dnf_info_packages_v0<'mw>(req: &mut Request, mut res: Response<'mw>) -> MiddlewareResult<'mw> {
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
fn project_list_v0<'mw>(req: &mut Request, mut res: Response<'mw>) -> MiddlewareResult<'mw> {
    let conn = req.db_conn().expect("Failed to get a database connection from the pool.");
    let mut project_list = Vec::new();
    let result = get_projects_name(&conn, "*");
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
fn project_info_v0<'mw>(req: &mut Request, mut res: Response<'mw>) -> MiddlewareResult<'mw> {
    let projects = req.param("projects").unwrap_or("").split(",");

    // Why does passing 'foo' match the route and passing: 'foo.1.1'
    // fail?

    let conn = req.db_conn().expect("Failed to get a database connection from the pool.");
    let mut project_info = Vec::new();
    for proj in projects {
        let result = get_projects_name(&conn, proj);
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

/// Enable CORS support
/// https://developer.mozilla.org/en-US/docs/Web/HTTP/Access_control_CORS
fn enable_cors<'mw>(_req: &mut Request, mut res: Response<'mw>) -> MiddlewareResult<'mw> {
    // Set appropriate headers
    res.set(header::AccessControlAllowOrigin::Any);
    res.set(header::AccessControlAllowHeaders(vec![
        // Hyper uses the `unicase::Unicase` type to ensure comparisons are done
        // case-insensitively. Here, we use `into()` to convert to one from a `&str`
        // so that we don't have to import the type ourselves.
        "Origin".into(),
        "X-Requested-With".into(),
        "Content-Type".into(),
        "Accept".into(),
    ]));

    // Pass control to the next middleware
    res.next_middleware()
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let program = &args[0];

    let mut opts = Options::new();
    opts.optflag("h", "help", "Show this usage message.");
// TODO: Set the host (default to 127.0.0.1)
// TODO: Set the port (default to 8000)

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(e) => { panic!(e.to_string()) }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    let db_path = if !matches.free.is_empty() {
        &matches.free[0]
    } else {
        print_usage(&program, opts);
        return;
    };

    let mut server = Nickel::new();

    // Use a pool of connections to the sqlite database
    let db_mgr = SqliteConnectionManager::new(db_path);
    let db_pool = Pool::new(Config::default(), db_mgr)
        .expect("Unable to initialize the connection pool.");
    server.utilize(SqliteMiddleware::with_pool(db_pool));

    server.utilize(enable_cors);

    server.get("/api/v0/test", test_v0);

    // Composer v0 API
    server.get("/api/v0/isos", unimplemented_v0);
    server.post("/api/v0/compose", unimplemented_v0);
    server.get("/api/v0/compose/status", unimplemented_v0);
    server.get("/api/v0/compose/status/:compose_id", unimplemented_v0);
    server.get("/api/v0/compose/types", compose_types_v0);
    server.get("/api/v0/compose/log/:kbytes", unimplemented_v0);
    server.post("/api/v0/compose/cancel", unimplemented_v0);

    server.get("/api/v0/dnf/transaction/:packages", unimplemented_v0);
    server.get("/api/v0/dnf/info/:packages", dnf_info_packages_v0);

    server.get("/api/v0/projects/list", project_list_v0);
    server.get("/api/v0/projects/info/:projects", project_info_v0);

    server.get("/api/v0/module/info/:modules", unimplemented_v0);
    // Is this first needed or will the 2nd just have an empty param?
    server.get("/api/v0/module/list", unimplemented_v0);
    server.get("/api/v0/module/list/:modules", unimplemented_v0);

    server.get("/api/v0/recipe/list", unimplemented_v0);
    server.get("/api/v0/recipe/:names", unimplemented_v0);
    server.post("/api/v0/recipe/:name", unimplemented_v0);

    server.listen("127.0.0.1:8000").unwrap();
}
