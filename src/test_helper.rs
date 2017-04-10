//! BDCS test helper functions

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

use rusqlite::{self, Connection};

use rpm::*;
use std::str::FromStr;

// structs and functions for creating in-memory databases
// id-less data types for creating test data
pub struct TestKeyValues {
    pub key_value: String,
    pub val_value: String,
    pub ext_value: Option<String>
}

pub struct TestFiles {
    pub path: String,
    pub file_user: String,
    pub file_group: String,
    pub mtime: i64,
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

pub fn create_test_db(data: &[TestData]) -> rusqlite::Result<Connection> {
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
            try!(conn.execute_named("
                insert into files (path, file_user, file_group, mtime)
                values (:path, :file_user, :file_group, :mtime)",
                &[(":path", &file.path),
                  (":file_user", &file.file_user),
                  (":file_group", &file.file_group),
                  (":mtime", &file.mtime)]));
            let file_id = conn.last_insert_rowid();

            for kv in &file.key_vals {
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

            for signature in &build.signatures {
                try!(insert_build_signature(conn, signature, build_id));
            }

            for file in &build.files {
                let file_id = try!(insert_file(conn, file));
                try!(conn.execute_named("insert into build_files (build_id, file_id) values (:build_id, :file_id)",
                                                &[(":build_id", &build_id), (":file_id", &file_id)]));
            }

            for kv in &build.key_vals {
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

            for build in &source.builds {
                try!(insert_build(conn, build, source_id));
            }

            for kv in &source.key_vals {
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

            for source in &project.sources {
                try!(insert_source(conn, source, project_id));
            }

            for kv in &project.key_vals {
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

            for file in &group.files {
                let file_id = try!(insert_file(conn, file));
                try!(conn.execute_named("insert into group_files (group_id, file_id) values (:group_id, :file_id)",
                                                &[(":group_id", &group_id), (":file_id", &file_id)]));
            }

            for child in &group.children {
                let child_id = try!(insert_group(conn, child));
                try!(conn.execute_named("insert into group_groups (parent_group_id, child_group_id) values (:parent_group_id, :child_group_id)",
                                                &[(":parent_group_id", &group_id), (":child_group_id", &child_id)]));
            }

            for kv in &group.key_vals {
                let key_val_id = try!(insert_key_val(conn, kv));
                try!(conn.execute_named("insert into group_key_values (group_id, key_val_id) values (:group_id, :key_val_id)",
                                                &[(":group_id", &group_id), (":key_val_id", &key_val_id)]));
            }

            for requirement in &group.requirements {
                let req_id = try!(insert_requirement(conn, requirement));
                try!(conn.execute_named("insert into group_requirements (group_id, req_id) values (:group_id, :req_id)",
                                                &[(":group_id", &group_id), (":req_id", &req_id)]));
            }

            Ok(group_id)
        }

        match *data {
            TestData::Projects(ref project) => try!(insert_project(conn, project)),
            TestData::Groups(ref group) => try!(insert_group(conn, group))
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

create table files (
    id integer primary key,
    path text not null,
    file_user text not null,
    file_group text not null,
    mtime integer not null
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

// higher-level structs and functions for package depsolve data
pub struct TestPkg {
    pub name: String,
    pub evr: EVR,
    pub arch: String,
    pub provides: Vec<Requirement>,
    pub requires: Vec<Requirement>,
    pub obsoletes: Vec<Requirement>,
    pub conflicts: Vec<Requirement>
}

// shorthand parse or panic Requirement
pub fn req(input: &str) -> Requirement {
    Requirement::from_str(input).unwrap()
}

// make generating a TestPkg easier
#[cfg_attr(feature="cargo-clippy", allow(too_many_arguments))]
pub fn testpkg(name: &str, epoch: Option<u32>, version: &str, release: &str, arch: &str,
               provides: &[&str], requires: &[&str], obsoletes: &[&str], conflicts: &[&str]) -> TestPkg {
    fn req_ref(input: &&str) -> Requirement { req(input) }
    TestPkg{name: name.to_string(), evr: EVR{epoch: epoch, version: version.to_string(), release: release.to_string()}, arch: arch.to_string(),
            provides: provides.iter().map(req_ref).collect(),
            requires: requires.iter().map(req_ref).collect(),
            obsoletes: obsoletes.iter().map(req_ref).collect(),
            conflicts: conflicts.iter().map(req_ref).collect()}
}

pub fn create_test_packages(data: &[TestPkg]) -> rusqlite::Result<Connection> {
    fn pkg_to_group(pkg: &TestPkg) -> TestData {
        let mut key_vals: Vec<TestKeyValues> = vec![
            TestKeyValues{key_value: "name".to_string(), val_value: pkg.name.clone(), ext_value: None},
            TestKeyValues{key_value: "version".to_string(), val_value: pkg.evr.version.clone(), ext_value: None},
            TestKeyValues{key_value: "release".to_string(), val_value: pkg.evr.release.clone(), ext_value: None},
            TestKeyValues{key_value: "arch".to_string(), val_value: pkg.arch.clone(), ext_value: None}];

        if let Some(epoch) = pkg.evr.epoch {
            key_vals.push(TestKeyValues{key_value: "epoch".to_string(), val_value: epoch.to_string(), ext_value: None});
        }

        for p in &pkg.provides {
            key_vals.push(TestKeyValues{key_value: "rpm-provide".to_string(), val_value: p.name.clone(), ext_value: Some(p.to_string())});
        }

        for o in &pkg.obsoletes {
            key_vals.push(TestKeyValues{key_value: "rpm-obsolete".to_string(), val_value: o.name.clone(), ext_value: Some(o.to_string())});
        }

        for c in &pkg.conflicts {
            key_vals.push(TestKeyValues{key_value: "rpm-conflict".to_string(), val_value: c.name.clone(), ext_value: Some(c.to_string())});
        }

        let requirements: Vec<TestRequirements> = pkg.requires.iter().map(|r| TestRequirements{req_language: "RPM".to_string(), req_context: "Runtime".to_string(),
                                                                                               req_strength: "Must".to_string(), req_expr: r.to_string()}).collect();

        TestData::Groups(TestGroups{name: pkg.name.clone(), group_type: "rpm".to_string(),
                                    files: vec![],
                                    children: vec![],
                                    key_vals: key_vals,
                                    requirements: requirements})
    }

    create_test_db(&data.iter().map(pkg_to_group).collect::<Vec<TestData>>())
}

// something like this should probably be in db.rs at some point
pub fn get_nevra_group_id(conn: &Connection, name: &str, epoch: Option<u32>, version: &str, release: &str, arch: &str) -> i64 {
    conn.query_row_named("
        select groups.id
        from groups
        join (group_key_values join key_val on group_key_values.key_val_id == key_val.id and key_val.key_value == 'name') name on groups.id == name.group_id
        join (group_key_values join key_val on group_key_values.key_val_id == key_val.id and key_val.key_value == 'version') ver  on groups.id == ver.group_id
        join (group_key_values join key_val on group_key_values.key_val_id == key_val.id and key_val.key_value == 'release') rel on groups.id == rel.group_id
        join (group_key_values join key_val on group_key_values.key_val_id == key_val.id and key_val.key_value == 'arch') arch on groups.id == arch.group_id
        left outer join (group_key_values join key_val on group_key_values.key_val_id == key_val.id and key_val.key_value == 'epoch') epoch on groups.id == epoch.group_id
        where name.val_value == :name and
              epoch.val_value is :epoch and
              ver.val_value  == :version and
              rel.val_value  == :release and
              arch.val_value == :arch
        ",
        &[(":name", &(name.to_string())),
          (":epoch", &(epoch.map(|e| e.to_string()))),
          (":version", &(version.to_string())),
          (":release", &(release.to_string())),
          (":arch", &(arch.to_string()))],
        |row| row.get(0)).unwrap()
}
