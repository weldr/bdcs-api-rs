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

use bdcs::depclose::*;
use bdcs::rpm::{Requirement, ReqOperator, EVR};
use bdcs::test_helper::*;
use rusqlite::Connection;

fn test_data() -> rusqlite::Result<Connection> {
    create_test_packages(&[
        // provides itself, doesn't require anything
        testpkg("singleton", None, "1.0", "1", "x86_64",
                &["singleton = 1.0-1"],
                &[],
                &[],
                &[])
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

    assert!(cmp_expression(&test_result, &close_dependencies(&conn, &["x86_64".to_string()], &["singleton".to_string()]).unwrap()));
}
