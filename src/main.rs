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

use std::env;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

// Database
use rusqlite::Connection;
use getopts::Options;
use r2d2::{Pool, Config};
use r2d2_sqlite::SqliteConnectionManager;

// API Framework
use nickel::{Nickel, MediaType, HttpRouter, Request, Response, MiddlewareResult, NickelError};
use nickel::status::StatusCode;
use nickel_sqlite::{SqliteMiddleware, SqliteRequestExtensions};
use hyper::header;
use unicase::UniCase;
use rustc_serialize::json;


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
    package_id: i64,
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

    server.get("/api/v0/module/info/:modules", unimplemented_v0);
    // Is this first needed or will the 2nd just have an empty param?
    server.get("/api/v0/module/list", unimplemented_v0);
    server.get("/api/v0/module/list/:modules", unimplemented_v0);

    server.get("/api/v0/recipe/list", unimplemented_v0);
    server.get("/api/v0/recipe/:names", unimplemented_v0);
    server.post("/api/v0/recipe/:name", unimplemented_v0);

    server.listen("127.0.0.1:8000").unwrap();
}
