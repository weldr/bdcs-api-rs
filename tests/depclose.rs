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
#[macro_use] extern crate pretty_assertions;

use bdcs::depclose::*;

#[test]
fn test_depexpression_display_atom() {
    assert_eq!(DepExpression::Atom(5).to_string(), "5");
}

#[test]
fn test_depexpression_display_not() {
    assert_eq!(DepExpression::Not(5).to_string(), "NOT 5");
}

#[test]
fn test_display_and() {
    assert_eq!(DepExpression::And(vec![]).to_string(), "()");
    assert_eq!(DepExpression::And(vec![DepExpression::Atom(5)]).to_string(), "(5)");
    assert_eq!(DepExpression::And(vec![DepExpression::Atom(5), DepExpression::Atom(6)]).to_string(), "(5 AND 6)");
}

#[test]
fn test_depexpression_display_or() {
    assert_eq!(DepExpression::Or(vec![]).to_string(), "()");
    assert_eq!(DepExpression::Or(vec![DepExpression::Atom(5)]).to_string(), "(5)");
    assert_eq!(DepExpression::Or(vec![DepExpression::Atom(5), DepExpression::Atom(6)]).to_string(), "(5 OR 6)");
}

#[test]
fn test_depexpression_display_combo() {
    assert_eq!(DepExpression::And(vec![DepExpression::And(vec![DepExpression::Atom(1), DepExpression::Atom(2), DepExpression::Atom(3)]),
                                       DepExpression::Atom(4),
                                       DepExpression::Or(vec![DepExpression::Atom(5), DepExpression::Atom(6)]),
                                       DepExpression::Or(vec![DepExpression::And(vec![DepExpression::Atom(7), DepExpression::Atom(8), DepExpression::Atom(9)]),
                                                              DepExpression::And(vec![DepExpression::Atom(10), DepExpression::Not(11)]),
                                                              DepExpression::Atom(12)]),
                                       DepExpression::Not(13)]).to_string(),
               "((1 AND 2 AND 3) AND 4 AND (5 OR 6) AND ((7 AND 8 AND 9) OR (10 AND NOT 11) OR 12) AND NOT 13)");
}
