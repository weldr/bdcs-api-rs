//! BDCS Sqlite Database Functions
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
use rusqlite::{self, Connection};
use std::path::{Path, PathBuf};


/// bdcs database schema structs
#[derive(Debug)]
pub struct Projects {
    pub id: i64,
    pub name: String,
    pub summary: String,
    pub description: String,
    pub homepage: Option<String>,
    pub upstream_vcs: String
}

#[derive(Debug)]
pub struct Sources {
    pub id: i64,
    pub project_id: i64,
    pub license: String,
    pub version: String,
    pub source_ref: String
}

#[derive(Debug)]
pub struct Builds {
    pub id: i64,
    pub source_id: i64,
    pub epoch: i64,
    pub release: String,
    pub arch: String,
    pub build_time: String,         // Should be Timespec or something like that
    pub changelog: Vec<u8>,
    pub build_config_ref: String,
    pub build_env_ref: String,
}

#[derive(Debug)]
pub struct BuildSignatures {
    pub id: i64,
    pub build_id: i64,
    pub signature_type: String,
    pub signature_data: Vec<u8>
}

#[derive(Debug)]
pub struct Files {
    pub id: i64,
    pub path: String,           // Could use rust's Path type?
    pub digest: String,
    pub file_type: String,
    pub file_mode: i64,
    pub file_user: String,
    pub file_group: String,
    pub file_size: i64,
    pub mtime: i64,
    pub symlink_target: Option<String>,
}

#[derive(Debug)]
pub enum FileAttrValues {
    file_id,
    attribute_type,
    attribute_value
}

#[derive(Debug)]
pub struct FileAttributes {
    pub id: i64,
    pub file_id: i64,
    pub attribute_type: String,
    pub attribute_value: String,
    pub FileIdKey: i64,
    pub TypeKey: FileAttrValues,
    pub XattrKey: FileAttrValues
}

#[derive(Debug)]
pub struct BuildFiles {
    pub id: i64,
    pub build_id: i64,
    pub file_id: i64
}

#[derive(Debug)]
pub struct KeyVal {
    pub id: i64,
    pub key_value: String,
    pub val_value: String
}

#[derive(Debug)]
pub struct ProjectKeyValues {
    pub id: i64,
    pub project_id: i64,
    pub key_val_id: i64
}

#[derive(Debug)]
pub struct SourceKeyValues {
    pub id: i64,
    pub source_id: i64,
    pub key_val_id: i64
}

#[derive(Debug)]
pub struct BuildKeyValues {
    pub id: i64,
    pub build_id: i64,
    pub key_val_id: i64
}

#[derive(Debug)]
pub struct FileKeyValues {
    pub id: i64,
    pub file_id: i64,
    pub key_val_id: i64
}

/// List contents of a package given by name.
pub fn get_pkg_files_name(conn: &Connection, pkgname: &str) -> rusqlite::Result<Vec<PathBuf>> {
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
pub fn get_pkg_files_nevra (conn: &Connection, pkgname: &str,
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
pub fn get_builds_name(conn: &Connection, project: &str) -> rusqlite::Result<Vec<Builds>> {
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
pub fn get_build_files(conn: &Connection, build_id: i64) -> rusqlite::Result<Vec<PathBuf>> {
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
pub fn get_builds_filename(conn: &Connection, filename: &str) -> rusqlite::Result<Vec<Builds>> {
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
pub fn get_projects_filename(conn: &Connection, filename: &str) -> rusqlite::Result<Vec<Projects>> {
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
pub fn get_projects_name(conn: &Connection, project: &str, offset: i64, limit: i64) -> rusqlite::Result<Vec<Projects>> {
    let mut stmt = try!(conn.prepare("
            select projects.*
            from projects
            where projects.name GLOB :project ORDER BY projects.id LIMIT :limit OFFSET :offset"));
    let mut rows = try!(stmt.query_named(&[(":project", &project), (":offset", &offset), (":limit", &limit)]));

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
pub fn get_source_id(conn: &Connection, source_id: i64) -> rusqlite::Result<Vec<Sources>> {
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
pub fn get_builds_project_id(conn: &Connection, project_id: i64) -> rusqlite::Result<Vec<Builds>> {
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
pub fn get_project_kv_project_id(conn: &Connection, project_id: i64) -> rusqlite::Result<Vec<KeyVal>> {
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
pub fn get_source_kv_source_id(conn: &Connection, source_id: i64) -> rusqlite::Result<Vec<KeyVal>> {
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
pub fn get_build_kv_build_id(conn: &Connection, build_id: i64) -> rusqlite::Result<Vec<KeyVal>> {
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
