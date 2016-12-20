//! BDCS Sqlite Database Functions
//!
// Copyright (C) 2016
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
//! ## BDCS database structs
//!
//! These structs are a 1:1 mapping of the sqlite tables used in the
//! bdcs sqlite database. The Int type maps to i64, Blob to Vec<u8>,
//! and everything else to String.
//!
//! ## TODO
//!
//! The database schema support should be versioned, with the ability
//! to upgrade older databases to newer schema.
//!

use rusqlite::{self, Connection};
use std::path::PathBuf;


/// High level details for upstream projects
#[derive(Debug,RustcEncodable)]
pub struct Projects {
    pub id: i64,
    pub name: String,
    pub summary: String,
    pub description: String,
    pub homepage: Option<String>,
    pub upstream_vcs: String
}

/// The location for source code used to build `Builds`
#[derive(Debug,RustcEncodable)]
pub struct Sources {
    pub id: i64,
    pub project_id: i64,
    pub license: String,
    pub version: String,
    pub source_ref: String
}

/// A specific build of a project
#[derive(Debug,RustcEncodable)]
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

/// Signatures verifying a build output
#[derive(Debug,RustcEncodable)]
pub struct BuildSignatures {
    pub id: i64,
    pub build_id: i64,
    pub signature_type: String,
    pub signature_data: Vec<u8>
}

/// Files created by a build
#[derive(Debug,RustcEncodable)]
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

/// File attribute types
#[derive(Debug,RustcEncodable)]
pub enum FileAttrValues {
    FileId,
    AttributeType,
    AttributeValue
}

/// Special attributes for files (eg. SELinux xattrs)
#[derive(Debug,RustcEncodable)]
pub struct FileAttributes {
    pub id: i64,
    pub file_id: i64,
    pub attribute_type: String,
    pub attribute_value: String,
    pub file_id_key: i64,
    pub type_key: FileAttrValues,
    pub xattr_key: FileAttrValues
}

/// The files associated with a specific entry from `Builds`
#[derive(Debug,RustcEncodable)]
pub struct BuildFiles {
    pub id: i64,
    pub build_id: i64,
    pub file_id: i64
}

/// A general key:value store
#[derive(Debug,RustcEncodable)]
pub struct KeyVal {
    pub id: i64,
    pub key_value: String,
    pub val_value: String
}

/// `Projects` related key:value
#[derive(Debug,RustcEncodable)]
pub struct ProjectKeyValues {
    pub id: i64,
    pub project_id: i64,
    pub key_val_id: i64
}

/// `Sources` related key:value
#[derive(Debug,RustcEncodable)]
pub struct SourceKeyValues {
    pub id: i64,
    pub source_id: i64,
    pub key_val_id: i64
}

/// `Builds` related key:value
#[derive(Debug,RustcEncodable)]
pub struct BuildKeyValues {
    pub id: i64,
    pub build_id: i64,
    pub key_val_id: i64
}

/// `Files` related key:value
#[derive(Debug,RustcEncodable)]
pub struct FileKeyValues {
    pub id: i64,
    pub file_id: i64,
    pub key_val_id: i64
}

/// Groups of projects
#[derive(Debug,RustcEncodable)]
pub struct Groups {
    pub id: i64,
    pub name: String,
    pub group_type: String
}

/// Files included in a `Groups`
#[derive(Debug,RustcEncodable)]
pub struct GroupFiles {
    pub id: i64,
    pub group_id: i64,
    pub file_id: i64
}

/// Groups of `Groups`
#[derive(Debug,RustcEncodable)]
pub struct GroupGroups {
    pub id: i64,
    pub parent_group_id: i64,
    pub child_group_id: i64
}

/// `Groups` related key:value
#[derive(Debug,RustcEncodable)]
pub struct GroupKeyValues {
    pub id: i64,
    pub group_id: i64,
    pub key_val_id: i64
}

/// Requirements
///
/// This describes how to determine what other projects or groups to include in the set of files to
/// be written.
///
#[derive(Debug,RustcEncodable)]
pub struct Requirements {
    pub id: i64,
    pub req_language: String,
    pub req_context: String,
    pub req_strength: String,
    pub req_expr: String
}

/// `Requirements` to use for specific `Groups` entries
#[derive(Debug,RustcEncodable)]
pub struct GroupRequirements {
    pub id: i64,
    pub group_id: i64,
    pub req_id: i64
}


/// List contents of a package given by name.
///
/// # Arguments
///
/// * `conn` - The database connection
/// * `pkgname` - The name of the package to search for, exact matches only.
///
/// # Returns
///
/// * A Vector of PathBuf entries containing the full path of all of the files included
///   in the package.
///
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
///
/// # Arguments
///
/// * `conn` - The database connection
/// * `pkgname` - The name of the package
/// * `epoch` - Epoch value, eg. 0
/// * `version` - Version string, eg. "1.2"
/// * `release` - Release string, eg. "1"
/// * `arch` - Architecture string, eg. "x86_64"
///
/// # Returns
///
/// * A Vector of PathBuf entries containing the full path of all of the files included
///   in the package version.
///
/// # Notes
///
/// This only matches the exact NEVRA.
///
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
///
/// # Arguments
///
/// * `conn` - The database connection
/// * `project` - The name of the project, exact matches only.
///
/// # Returns
///
/// * A Vector of [Builds](struct.Builds.html) for the matching project name.
///
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
///
/// # Arguments
///
/// * `conn` - The database connection
/// * `build_id` - The id of the [Builds](struct.Builds.html) entry to reference
///
/// # Returns
///
/// * A Vector of PathBuf entries containing the full path of all of the files included
///   in the package version.
///
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
///
/// # Arguments
///
/// * `conn` - The database connection
/// * `filename` - The full path of the file to match
///
/// # Returns
///
/// * A Vector of [Builds](struct.Builds.html) for the matching project name.
///
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
///
/// # Arguments
///
/// * `conn` - The database connection
/// * `filename` - The full path of the file to match
///
/// # Returns
///
/// * A Vector of [Projects](struct.Projects.html) for the matching filename.
///
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
///
/// # Arguments
///
/// * `conn` - The database connection
/// * `project` - The name of the project, glob search patterns allowed
/// * `offset` - Number of results to skip before returning `limit`
/// * `limit` - Maximum number of results to return
///
/// # Returns
///
/// * A Vector of [Projects](struct.Projects.html) for the matching project name/glob
///
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
///
/// # Arguments
///
/// * `conn` - The database connection
/// * `source_id` - The id of the [Sources](struct.Sources.html) entry to get
///
/// # Returns
///
/// * A Vector of [Sources](struct.Sources.html) for the matching `source_id`
///
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
///
/// # Arguments
///
/// * `conn` - The database connection
/// * `project_id` - The id of the [Projects](struct.Projects.html) entry to get
///
/// # Returns
///
/// * A Vector of [Builds](struct.Builds.html) for the matching `project_id`
///
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


/// Get key:value data for the project based on project id
///
/// # Arguments
///
/// * `conn` - The database connection
/// * `project_id` - The id of the [Projects](struct.Projects.html) entry to get
///
/// # Returns
///
/// * A Vector of [KeyVal](struct.KeyVal.html) for the matching `project_id`
///
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

/// Get key:value data for the sources based on source id
///
/// # Arguments
///
/// * `conn` - The database connection
/// * `source_id` - The id of the [Sources](struct.Sources.html) entry to get
///
/// # Returns
///
/// * A Vector of [KeyVal](struct.KeyVal.html) for the matching `source_id`
///
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


/// Get key:value data for the builds based on build id
///
/// # Arguments
///
/// * `conn` - The database connection
/// * `build_id` - The id of the [Builds](struct.Builds.html) entry to get
///
/// # Returns
///
/// * A Vector of [KeyVal](struct.KeyVal.html) for the matching `build_id`
///
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


/// Find all groups matching a group name
///
/// # Arguments
///
/// * `conn` - The database connection
/// * `group` - The name of the group, glob search patterns allowed
/// * `offset` - Number of results to skip before returning `limit`
/// * `limit` - Maximum number of results to return
///
/// # Returns
///
/// * A Vector of [Groups](struct.Groups.html) for the matching group name/glob
///
pub fn get_groups_name(conn: &Connection, group: &str, offset: i64, limit: i64) -> rusqlite::Result<Vec<Groups>> {
    let mut stmt = try!(conn.prepare("
            select groups.*
            from groups
            where groups.name GLOB :group ORDER BY groups.id LIMIT :limit OFFSET :offset"));
    let mut rows = try!(stmt.query_named(&[(":group", &group), (":offset", &offset), (":limit", &limit)]));

    let mut contents = Vec::new();
    while let Some(row) = rows.next() {
        let row = try!(row);
        // Sure would be nice not to use indexes here!
        contents.push(Groups {
                        id: row.get(0),
                        name: row.get(1),
                        group_type: row.get(2),
                    });
    }
    Ok(contents)
}


/// Get key:value data for the groups based on group id
///
/// # Arguments
///
/// * `conn` - The database connection
/// * `group_id` - The id of the [Groups](struct.Groups.html) entry to get
///
/// # Returns
///
/// * A Vector of [KeyVal](struct.KeyVal.html) for the matching `group_id`
///
pub fn get_groups_kv_group_id(conn: &Connection, group_id: i64) -> rusqlite::Result<Vec<KeyVal>> {
    let mut stmt = try!(conn.prepare("
            select key_val.*
            from group_key_values, key_val
            on key_val.id == group_key_values.key_val_id
            where group_key_values.group_id == :group_id"));
    let mut rows = try!(stmt.query_named(&[(":group_id", &group_id)]));

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


/// Get group requirements
///
/// # Arguments
///
/// * `conn` - The database connection
/// * `group_id` - The id of the [Groups](struct.Groups.html) entry to get
///
/// # Returns
///
/// * A Vector of [Requirements](struct.Requirements.html) for the matching `group_id`
///
pub fn get_requirements_group_id(conn: &Connection, group_id: i64) -> rusqlite::Result<Vec<Requirements>> {
    let mut stmt = try!(conn.prepare("
            select requirements.*
            from group_requirements, requirements
            on requirements.id == group_requirements.req_id
            where group_requirements.group_id == :group_id"));
    let mut rows = try!(stmt.query_named(&[(":group_id", &group_id)]));

    let mut contents = Vec::new();
    while let Some(row) = rows.next() {
        let row = try!(row);
        // Sure would be nice not to use indexes here!
        contents.push(Requirements {
                        id: row.get(0),
                        req_language: row.get(1),
                        req_context: row.get(2),
                        req_strength: row.get(3),
                        req_expr: row.get(4),
                    });
    }
    Ok(contents)
}
