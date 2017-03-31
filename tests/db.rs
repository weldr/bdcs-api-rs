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

use bdcs::db::*;
use bdcs::test_helper::*;
use std::path::PathBuf;

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
fn test_db_1() -> rusqlite::Result<Connection> {
    create_test_db(&[
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
    let conn = create_test_db(&[]).unwrap();
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
    create_test_db(&[
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
    let conn = create_test_db(&[]).unwrap();
    let test_data: Vec<PathBuf> = vec![];
    assert_eq_no_order!(get_pkg_files_nevra(&conn, "whatever", 0, "0", "0", "x86_64").unwrap(), test_data);
}

#[test]
fn test_data_3() -> () {
    let conn = test_db_2().unwrap();
    let test_data = vec![PathBuf::from("/r1/1")];
    assert_eq_no_order!(get_pkg_files_nevra(&conn, "project-one", 1, "1.47", "1", "x86_64").unwrap(), test_data);
}

fn test_db_3() -> rusqlite::Result<Connection> {
    create_test_db(&[
                   TestData::Groups(
                       TestGroups{name: "test-package".to_string(),
                                  group_type: "rpm".to_string(),
                                  files: vec![],
                                  children: vec![],
                                  key_vals: vec![
                                      TestKeyValues{key_value: "name".to_string(),
                                                    val_value: "test-package".to_string(),
                                                    ext_value: None}
                                  ],
                                  requirements: vec![]})
    ])
}

#[test]
fn test_get_group_by_name_match() {
    let conn = test_db_3().unwrap();
    assert_eq!(get_groups_by_name(&conn, "test-package", "rpm").unwrap().len(), 1);
}

#[test]
fn test_get_group_by_name_empty() {
    let conn = test_db_3().unwrap();
    assert_eq!(get_groups_by_name(&conn, "no-package", "rpm").unwrap().len(), 0);
}
