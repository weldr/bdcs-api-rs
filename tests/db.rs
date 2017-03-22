//! DB test and test helper functions

// Copyright (C) 2017 Red Hat, Inc.
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

extern crate bdcs;
extern crate rusqlite;

use rusqlite::Connection;
use std::collections::HashSet;

// id-less data types for creating test data
pub struct TestKeyValues {
    pub key_value: String,
    pub val_value: String,
    pub ext_value: Option<String>
}

pub struct TestFiles {
    pub path: String,
    pub digest: String,
    pub file_type: String,
    pub file_mode: i64,
    pub file_user: String,
    pub file_group: String,
    pub file_size: i64,
    pub mtime: i64,
    pub symlink_target: Option<String>,
    pub key_vals: Vec<TestKeyValues>
}

pub struct TestBuildSignatures {
    pub signature_type: String,
    pub signature_data: Vec<u8>
}

pub struct TestBuilds {
    pub epoch: i64,
    pub release: String,
    pub arch: String,
    pub build_time: String,
    pub changelog: Vec<u8>,
    pub build_config_ref: String,
    pub build_env_ref: String,
    pub signatures: Vec<TestBuildSignatures>,
    pub files: Vec<TestFiles>,
    pub key_vals: Vec<TestKeyValues>
}

pub struct TestSources {
    pub license: String,
    pub version: String,
    pub source_ref: String,
    pub builds: Vec<TestBuilds>,
    pub key_vals: Vec<TestKeyValues>
}

pub struct TestProjects {
    pub name: String,
    pub summary: String,
    pub description: String,
    pub homepage: Option<String>,
    pub upstream_vcs: String,
    pub sources: Vec<TestSources>,
    pub key_vals: Vec<TestKeyValues>
}

pub struct TestRequirements {
    pub req_language: String,
    pub req_context: String,
    pub req_strength: String,
    pub req_expr: String
}

pub struct TestGroups {
    pub name: String,
    pub group_type: String,
    pub files: Vec<TestFiles>,
    pub children: Vec<Box<TestGroups>>,
    pub key_vals: Vec<TestKeyValues>,
    pub requirements: Vec<TestRequirements>
}

// combo type
pub enum TestData {
    Projects(TestProjects),
    Groups(TestGroups)
}

pub fn create_test_db(data: Vec<TestData>) -> rusqlite::Result<Connection> {
    fn insert_record(conn: &Connection, data: &TestData) -> rusqlite::Result<()> {
        fn insert_key_val(conn: &Connection, keyval: &TestKeyValues) -> rusqlite::Result<i64> {
            try!(conn.execute_named("
                insert into key_val (key_value, val_value, ext_value)
                values (:key_value, :val_value, :ext_value)",
                &[(":key_value", &keyval.key_value),
                  (":val_value", &keyval.val_value),
                  (":ext_value", &keyval.ext_value)]));
            Ok(conn.last_insert_rowid())
        }

        fn insert_file(conn: &Connection, file: &TestFiles) -> rusqlite::Result<i64> {
            fn get_file_type_id(conn: &Connection, file_type: &String) -> rusqlite::Result<i64> {
                conn.query_row_named("select id from file_types where file_type == :file_type",
                                     &[(":file_type", file_type)],
                                     |row| row.get(0) )
            }

            try!(conn.execute_named("
                insert into files (path, digest, file_type_id, file_mode, file_user, file_group, file_size, mtime, symlink_target)
                values (:path, :digest, :file_type_id, :file_mode, :file_user, :file_group, :file_size, :mtime, :symlink_target)",
                &[(":path", &file.path),
                  (":digest", &file.digest),
                  (":file_type_id", &try!(get_file_type_id(conn, &file.file_type))),
                  (":file_mode", &file.file_mode),
                  (":file_user", &file.file_user),
                  (":file_group", &file.file_group),
                  (":file_size", &file.file_size),
                  (":mtime", &file.mtime),
                  (":symlink_target", &file.symlink_target)]));
            let file_id = conn.last_insert_rowid();

            for kv in file.key_vals.iter() {
                let key_val_id = try!(insert_key_val(conn, kv));
                try!(conn.execute_named("
                    insert into file_key_values (file_id, key_val_id) values (:file_id, :key_val_id)",
                    &[(":file_id", &file_id), (":key_val_id", &key_val_id)]));
            }
            Ok(file_id)
        }

        fn insert_build_signature(conn: &Connection, build_signature: &TestBuildSignatures, build_id: i64) -> rusqlite::Result<i64> {
            try!(conn.execute_named("
                insert into build_signatures (build_id, signature_type, signature_data)
                values (:build_id, :signature_type, :signature_data)",
                &[(":build_id", &build_id),
                  (":signature_type", &build_signature.signature_type),
                  (":signature_data", &build_signature.signature_data)]));
            Ok(conn.last_insert_rowid())
        }

        fn insert_build(conn: &Connection, build: &TestBuilds, source_id: i64) -> rusqlite::Result<i64> {
            try!(conn.execute_named("
                insert into builds (source_id, epoch, release, arch, build_time, changelog, build_config_ref, build_env_ref)
                values (:source_id, :epoch, :release, :arch, :build_time, :changelog, :build_config_ref, :build_env_ref)",
                &[(":source_id", &source_id),
                (":epoch", &build.epoch),
                (":release", &build.release),
                (":arch", &build.arch),
                (":build_time", &build.build_time),
                (":changelog", &build.changelog),
                (":build_config_ref", &build.build_config_ref),
                (":build_env_ref", &build.build_env_ref)]));
            let build_id = conn.last_insert_rowid();

            for signature in build.signatures.iter() {
                try!(insert_build_signature(conn, signature, build_id));
            }

            for file in build.files.iter() {
                let file_id = try!(insert_file(conn, file));
                try!(conn.execute_named("insert into build_files (build_id, file_id) values (:build_id, :file_id)",
                                                &[(":build_id", &build_id), (":file_id", &file_id)]));
            }

            for kv in build.key_vals.iter() {
                let key_val_id = try!(insert_key_val(conn, kv));
                try!(conn.execute_named("insert into build_values (build_id, key_val_id) values (:build_id, :key_val_id)",
                                                &[(":build_id", &build_id), (":key_val_id", &key_val_id)]));
            }

            Ok(build_id)
        }

        fn insert_source(conn: &Connection, source: &TestSources, project_id: i64) -> rusqlite::Result<i64> {
            try!(conn.execute_named("
                insert into sources (project_id, license, version, source_ref)
                values (:project_id, :license, :version, :source_ref)",
                &[(":project_id", &project_id),
                  (":license", &source.license),
                  (":version", &source.version),
                  (":source_ref", &source.source_ref)]));
            let source_id = conn.last_insert_rowid();

            for build in source.builds.iter() {
                try!(insert_build(conn, build, source_id));
            }

            for kv in source.key_vals.iter() {
                let key_val_id = try!(insert_key_val(conn, kv));
                try!(conn.execute_named("insert into source_key_values (source_id, key_val_id) values (:source_id, :key_val_id)",
                                                &[(":source_id", &source_id), (":key_val_id", &key_val_id)]));
            }

            Ok(source_id)
        }

        fn insert_project(conn: &Connection, project: &TestProjects) -> rusqlite::Result<i64> {
            try!(conn.execute_named("
                insert into projects (name, summary, description, homepage, upstream_vcs)
                values (:name, :summary, :description, :homepage, :upstream_vcs)",
                &[(":name", &project.name),
                  (":summary", &project.summary),
                  (":description", &project.description),
                  (":homepage", &project.homepage),
                  (":upstream_vcs", &project.upstream_vcs)]));
            let project_id = conn.last_insert_rowid();

            for source in project.sources.iter() {
                try!(insert_source(conn, source, project_id));
            }

            for kv in project.key_vals.iter() {
                let key_val_id = try!(insert_key_val(conn, kv));
                try!(conn.execute_named("insert into project_values (project_id, key_val_id) values (:project_id, :key_val_id)",
                                                &[(":project_id", &project_id), (":key_val_id", &key_val_id)]));
            }

            Ok(project_id)
        }

        fn insert_requirement(conn: &Connection, requirement: &TestRequirements) -> rusqlite::Result<i64> {
            try!(conn.execute_named("
                insert into requirements (req_language, req_context, req_strength, req_expr)
                values (:req_language, :req_context, :req_strength, :req_expr)",
                &[(":req_language", &requirement.req_language),
                  (":req_context", &requirement.req_context),
                  (":req_strength", &requirement.req_strength),
                  (":req_expr", &requirement.req_expr)]));
            Ok(conn.last_insert_rowid())
        }

        fn insert_group(conn: &Connection, group: &TestGroups) -> rusqlite::Result<i64> {
            try!(conn.execute_named("
                insert into groups (name, group_type) values (:name, :group_type)",
                &[(":name", &group.name), (":group_type", &group.group_type)]));
            let group_id = conn.last_insert_rowid();

            for file in group.files.iter() {
                let file_id = try!(insert_file(conn, file));
                try!(conn.execute_named("insert into group_files (group_id, file_id) values (:group_id, :file_id)",
                                                &[(":group_id", &group_id), (":file_id", &file_id)]));
            }

            for child in group.children.iter() {
                let child_id = try!(insert_group(conn, child));
                try!(conn.execute_named("insert into group_groups (parent_group_id, child_group_id) values (:parent_group_id, :child_group_id)",
                                                &[(":parent_group_id", &group_id), (":child_group_id", &child_id)]));
            }

            for kv in group.key_vals.iter() {
                let key_val_id = try!(insert_key_val(conn, kv));
                try!(conn.execute_named("insert into group_key_values (group_id, key_val_id) values (:group_id, :key_val_id)",
                                                &[(":group_id", &group_id), (":key_val_id", &key_val_id)]));
            }

            for requirement in group.requirements.iter() {
                let req_id = try!(insert_requirement(conn, requirement));
                try!(conn.execute_named("insert into group_requirements (group_id, req_id) values (:group_id, :req_id)",
                                                &[(":group_id", &group_id), (":req_id", &req_id)]));
            }

            Ok(group_id)
        }

        match data {
            &TestData::Projects(ref project) => try!(insert_project(conn, project)),
            &TestData::Groups(ref group) => try!(insert_group(conn, group))
        };

        Ok(())
    }

    let conn = try!(Connection::open_in_memory());

    // copied from bdcs/schema.sql
    try!(conn.execute_batch("
BEGIN;
create table projects (
    id integer primary key,
    name text not null unique,
    summary text not null,
    description text not null,
    homepage text,
    upstream_vcs text not null
);

create table sources (
    id integer primary key,
    project_id integer references projects(id) not null,
    license text not null,
    version text not null,
    source_ref text not null
);
create index sources_project_id_idx on sources(project_id);

create table builds (
    id integer primary key,
    source_id integer references sources(id) not null,
    epoch integer default 0,
    release text not null,
    arch text not null,
    build_time text not null,
    changelog blob not null,
    build_config_ref text not null,
    build_env_ref text not null
);
create index builds_source_id_idx on builds(source_id);

create table build_signatures (
    id integer primary key,
    build_id integer references build(id) not null,
    signature_type text not null,
    signature_data blob not null
);
create index build_signatures_build_id_idx on build_signatures(build_id);

create table file_types (
    id integer primary key,
    file_type text not null
);
insert into file_types (file_type) values
    ('regular file'),
    ('directory'),
    ('socket'),
    ('symbolic link'),
    ('block device'),
    ('character device'),
    ('FIFO');

create table files (
    id integer primary key,
    path text not null,
    digest text not null,
    file_type_id integer references file_types(id) not null,
    file_mode integer not null,
    file_user text not null,
    file_group text not null,
    file_size integer not null,
    mtime integer not null,
    symlink_target text
);
create index files_path_idx on files(path);

create table build_files (
    id integer primary key,
    build_id integer references build(id) not null,
    file_id integer references files(id) not null
);
create index build_files_build_id_idx on build_files(build_id);
create index build_files_file_id_idx on build_files(file_id);

create table key_val (
    id integer primary key,
    key_value text not null,
    val_value text not null,
    ext_value text
);

create index key_val_key_value_idx on key_val(key_value);
create index key_val_val_value_idx on key_val(key_value, val_value);

create table project_values (
    id integer primary key,
    project_id integer references projects(id) not null,
    key_val_id integer references key_val(id) not null
);
create index project_values_project_id_idx on project_values(project_id);
create index project_values_key_val_id_idx on project_values(key_val_id);

create table source_key_values (
    id integer primary key,
    source_id integer references sources(id) not null,
    key_val_id integer references key_val(id) not null
);
create index source_key_values_source_id_idx on source_key_values(source_id);
create index source_key_values_key_val_id_idx on source_key_values(key_val_id);

create table build_key_values (
    id integer primary key,
    build_id integer references builds(id) not null,
    key_val_id integer references key_val(id) not null
);
create index build_key_values_build_id_idx on build_key_values(build_id);
create index build_key_values_key_val_id_idx on build_key_values(key_val_id);

create table file_key_values (
    id integer primary key,
    file_id integer references files(id) not null,
    key_val_id integer references key_val(id) not null
);
create index file_key_values_file_id_idx on file_key_values(file_id);
create index file_key_values_key_val_id_idx on file_key_values(key_val_id);

create table groups (
    id integer primary key,
    name text not null,
    group_type text not null
);
create index groups_name_idx on groups(name);

create table group_files (
    id integer primary key,
    group_id integer references groups(id) not null,
    file_id integer references files(id) not null
);
create index group_files_group_id_idx on group_files(group_id);
create index group_files_file_id_idx on group_files(file_id);

create table group_groups (
    id integer primary key,
    parent_group_id references groups(id) not null,
    child_group_id references groups(id) not null
);
create index group_groups_parent_group_id_idx on group_groups(parent_group_id);
create index group_groups_child_group_id_idx on group_groups(child_group_id);

create table group_key_values (
    id integer primary key,
    group_id integer references groups(id) not null,
    key_val_id integer references key_val(id) not null
);
create index group_key_values_group_id_idx on group_key_values(group_id);
create index group_key_values_key_val_id_idx on group_key_values(key_val_id);

create table requirements (
    id integer primary key,
    req_language text not null,
    req_context text not null,
    req_strength text not null,
    req_expr text not null
);

create table group_requirements (
    id integer primary key,
    group_id integer references groups(id) not null,
    req_id integer references requirements(id) not null
);
create index group_requirements_group_id_idx on group_requirements(group_id);
create index group_requirements_req_id_idx on group_requirements(req_id);
COMMIT;
"));

    for rec in data.iter() {
        try!(insert_record(&conn, rec));
    }

    Ok(conn)
}

// order doesn't usually matter for comparing database results, so take that out of the equation
macro_rules! assert_eq_no_order {
    ($a:expr, $b:expr) => {
        {
            let v1: Vec<_> = $a;
            let v2: Vec<_> = $b;
            assert_eq!(v1.iter().collect::<HashSet<_>>(),
                       v2.iter().collect::<HashSet<_>>());
        }
    }
}

// get_pkg_files_name: return a list of files associated with a given packageName=<val> key/val pair
use bdcs::db::*;
use std::path::PathBuf;

fn test_db_1() -> rusqlite::Result<Connection> {
    create_test_db(vec![
                   TestData::Groups(TestGroups{name: "group-one".to_string(),
                                               group_type: "rpm".to_string(),
                                               files: vec![TestFiles{path: "/one/1".to_string(), digest: "".to_string(), file_type: "regular file".to_string(),
                                                                     file_mode: 0o644, file_user: "".to_string(), file_group: "".to_string(), file_size: 0,
                                                                     mtime: 0, symlink_target: None,
                                                                     key_vals: vec![TestKeyValues{key_value: "packageName".to_string(), val_value: "group-one".to_string(), ext_value: None}]},
                                                           TestFiles{path: "/one/2".to_string(), digest: "".to_string(), file_type: "regular file".to_string(),
                                                                     file_mode: 0o644, file_user: "".to_string(), file_group: "".to_string(), file_size: 0,
                                                                     mtime: 0, symlink_target: None,
                                                                     key_vals: vec![TestKeyValues{key_value: "packageName".to_string(), val_value: "group-one".to_string(), ext_value: None}]}],
                                               children: vec![],
                                               key_vals: vec![],
                                               requirements: vec![]}),
                   TestData::Groups(TestGroups{name: "group-two".to_string(),
                                               group_type: "rpm".to_string(),
                                               files: vec![TestFiles{path: "/two/1".to_string(), digest: "".to_string(), file_type: "regular file".to_string(),
                                                                     file_mode: 0o644, file_user: "".to_string(), file_group: "".to_string(), file_size: 0,
                                                                     mtime: 0, symlink_target: None,
                                                                     key_vals: vec![TestKeyValues{key_value: "packageName".to_string(), val_value: "group-two".to_string(), ext_value: None}]},
                                                           TestFiles{path: "/two/2".to_string(), digest: "".to_string(), file_type: "regular file".to_string(),
                                                                     file_mode: 0o644, file_user: "".to_string(), file_group: "".to_string(), file_size: 0,
                                                                     mtime: 0, symlink_target: None,
                                                                     key_vals: vec![TestKeyValues{key_value: "packageName".to_string(), val_value: "group-two".to_string(), ext_value: None}]}],
                                               children: vec![],
                                               key_vals: vec![],
                                               requirements: vec![]})
                       ])
}

#[test]
fn test_get_pkg_empty() -> () {
    let conn = create_test_db(vec![]).unwrap();
    let test_data: Vec<PathBuf> = vec![];
    assert_eq!(get_pkg_files_name(&conn, "whatever").unwrap(), test_data);
}

#[test]
fn test_data_1() -> () {
    let conn = test_db_1().unwrap();
    let test_data = vec![PathBuf::from("/one/1"), PathBuf::from("/one/2")];
    let test_result = get_pkg_files_name(&conn, "group-one").unwrap();

    assert_eq_no_order!(test_data, test_result);
}

#[test]
fn test_data_2() -> () {
    let conn = test_db_1().unwrap();
    let test_data = vec![PathBuf::from("/two/1"), PathBuf::from("/two/2")];
    let test_result = get_pkg_files_name(&conn, "group-two").unwrap();

    assert_eq_no_order!(test_data, test_result);
}

// get_pkg_files_nevra: return a list of files associated with a given build, selected by the NEVRA of the build
fn test_db_2() -> rusqlite::Result<Connection> {
    create_test_db(vec![
                   TestData::Projects(TestProjects{name: "project-one".to_string(), summary: "".to_string(), description: "".to_string(),
                                                   homepage: None, upstream_vcs: "".to_string(), key_vals: vec![],
                                                   sources: vec![
                                                       TestSources{version: "1.47".to_string(),
                                                                   license: "".to_string(), source_ref: "".to_string(), key_vals: vec![],
                                                                   builds: vec![
                                                                       TestBuilds{epoch: 1, release: "1".to_string(), arch: "x86_64".to_string(),
                                                                                  build_time: "".to_string(), changelog: vec![], build_config_ref: "".to_string(),
                                                                                  build_env_ref: "".to_string(), signatures: vec![], key_vals: vec![],
                                                                                  files: vec![
                                                                                      TestFiles{path: "/r1/1".to_string(), digest: "".to_string(), file_type: "regular file".to_string(),
                                                                                                file_mode: 0o644, file_user: "".to_string(), file_group: "".to_string(), file_size: 0,
                                                                                                mtime: 0, symlink_target: None,
                                                                                                key_vals: vec![TestKeyValues{key_value: "packageName".to_string(), val_value: "project-one".to_string(), ext_value: None}]}
                                                                                              ]}
                                                                               ]}
                                                                ]})
                       ])
}

#[test]
fn test_empty() -> () {
    let conn = create_test_db(vec![]).unwrap();
    let test_data: Vec<PathBuf> = vec![];
    assert_eq_no_order!(get_pkg_files_nevra(&conn, "whatever", 0, "0", "0", "x86_64").unwrap(), test_data);
}

#[test]
fn test_data_3() -> () {
    let conn = test_db_2().unwrap();
    let test_data = vec![PathBuf::from("/r1/1")];
    assert_eq_no_order!(get_pkg_files_nevra(&conn, "project-one", 1, "1.47", "1", "x86_64").unwrap(), test_data);
}
