//! depclose test and helper functions

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

use std::str::FromStr;
use std::rc::Rc;

use bdcs::depclose::*;

mod db;

use db::*;
use bdcs::rpm::{Requirement, ReqOperator, EVR};
use rusqlite::Connection;

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
pub fn testpkg(name: &str, epoch: Option<u32>, version: &str, release: &str, arch: &str,
               provides: Vec<&str>, requires: Vec<&str>, obsoletes: Vec<&str>, conflicts: Vec<&str>) -> TestPkg {
    fn req_ref(input: &&str) -> Requirement { req(input) }
    TestPkg{name: name.to_string(), evr: EVR{epoch: epoch, version: version.to_string(), release: release.to_string()}, arch: arch.to_string(),
            provides: provides.iter().map(req_ref).collect(),
            requires: requires.iter().map(req_ref).collect(),
            obsoletes: obsoletes.iter().map(req_ref).collect(),
            conflicts: conflicts.iter().map(req_ref).collect()}
}

pub fn create_test_packages(data: Vec<TestPkg>) -> rusqlite::Result<Connection> {
    fn pkg_to_group(pkg: &TestPkg) -> TestData {
        let mut key_vals: Vec<TestKeyValues> = vec![
            TestKeyValues{key_value: "name".to_string(), val_value: pkg.name.clone(), ext_value: None},
            TestKeyValues{key_value: "version".to_string(), val_value: pkg.evr.version.clone(), ext_value: None},
            TestKeyValues{key_value: "release".to_string(), val_value: pkg.evr.release.clone(), ext_value: None},
            TestKeyValues{key_value: "arch".to_string(), val_value: pkg.arch.clone(), ext_value: None}];

        if let Some(epoch) = pkg.evr.epoch {
            key_vals.push(TestKeyValues{key_value: "epoch".to_string(), val_value: epoch.to_string(), ext_value: None});
        }

        for p in pkg.provides.iter() {
            key_vals.push(TestKeyValues{key_value: "rpm-provide".to_string(), val_value: p.name.clone(), ext_value: Some(p.to_string())});
        }

        for o in pkg.obsoletes.iter() {
            key_vals.push(TestKeyValues{key_value: "rpm-obsolete".to_string(), val_value: o.name.clone(), ext_value: Some(o.to_string())});
        }

        for c in pkg.conflicts.iter() {
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

    create_test_db(data.iter().map(pkg_to_group).collect())
}

// O(n^2) cmp function because RefCell makes everything terrible
pub fn cmp_expression(e1: &DepExpression, e2: &DepExpression) -> bool {
    match (e1, e2) {
        (&DepExpression::Atom(ref a1), &DepExpression::Atom(ref a2)) => a1 == a2,
        (&DepExpression::Not(ref n1), &DepExpression::Not(ref n2)) => {
            let e_sub_1 = n1.borrow();
            let e_sub_2 = n2.borrow();
            cmp_expression(&e_sub_1, &e_sub_2)
        },
        (&DepExpression::And(ref lst1), &DepExpression::And(ref lst2)) |
        (&DepExpression::Or(ref lst1), &DepExpression::Or(ref lst2)) => {
            if lst1.len() != lst2.len() {
                return false;
            }

            for r1 in lst1.iter() {
                let l1 = r1.borrow();
                if !lst2.iter().any(|r2| {
                    let l2 = r2.borrow();
                    cmp_expression(&l1, &l2)
                }) {
                    return false;
                }
            }

            true
        },
        _ => false
    }
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

// shorthand for boxing up a type
fn rc(expr: DepExpression) -> Rc<DepCell<DepExpression>> {
    Rc::new(DepCell::new(expr))
}

fn test_data() -> rusqlite::Result<Connection> {
    create_test_packages(vec![
        // provides itself, doesn't require anything
        testpkg("singleton", None, "1.0", "1", "x86_64",
                vec!["singleton = 1.0-1"],
                vec![],
                vec![],
                vec![])
    ])
}

#[test]
fn test_singleton() -> () {
    let conn = test_data().unwrap();
    let test_result =
        DepExpression::And(vec![            // And of all packages requested
            rc(DepExpression::Or(vec![      // Or of each package matching a single name
                rc(DepExpression::And(vec![ // actual package
                    // self
                    rc(DepExpression::Atom(DepAtom::GroupId(get_nevra_group_id(&conn, "singleton", None, "1.0", "1", "x86_64")))),
                    // provides
                    rc(DepExpression::And(vec![
                        rc(DepExpression::Atom(DepAtom::Requirement(req("singleton = 1.0-1")))),
                        // special one for Obsoletes matches
                        rc(DepExpression::Atom(DepAtom::Requirement(Requirement{name: "PKG: singleton".to_string(),
                                                                                expr: Some((ReqOperator::EqualTo, EVR::from_str("1.0").unwrap()))})))
                    ]))
                ]))
            ]))
        ]);

    assert!(cmp_expression(&test_result, &close_dependencies(&conn, &vec!["x86_64".to_string()], &vec!["singleton".to_string()]).unwrap()));
}
