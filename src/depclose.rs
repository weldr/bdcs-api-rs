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

use db::*;
use rpm::*;
use rusqlite::Connection;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::str::FromStr;
use itertools::Itertools;
use std::rc::Rc;
use std::cell::{Cell, RefCell};
use std::ops::Deref;

// TODO might need to mess with the type for depsolve
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum DepAtom {
    GroupId(i64),
    Requirement(Requirement)
}

impl fmt::Display for DepAtom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &DepAtom::GroupId(i)            => write!(f, "groupid={}", i),
            &DepAtom::Requirement(ref r)    => write!(f, "({})", r)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DepExpression {
    Atom(DepAtom),
    And(Vec<Rc<DepCell<DepExpression>>>),
    Or(Vec<Rc<DepCell<DepExpression>>>),
    Not(Rc<DepCell<DepExpression>>)
}

impl fmt::Display for DepExpression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &DepExpression::Atom(ref req)    => write!(f, "{}", req),
            &DepExpression::And(ref lst)     => { let strs: String = lst.iter().map(|x| x.borrow().to_string()).intersperse(String::from(" AND ")).collect();
                                                  write!(f, "{}", strs)
                                                },
            &DepExpression::Or(ref lst)      => { let strs: String = lst.iter().map(|x| x.borrow().to_string()).intersperse(String::from(" OR ")).collect();
                                                  write!(f, "{}", strs)
                                                },
            &DepExpression::Not(ref expr)    => write!(f, "NOT {}", *(expr.borrow()))
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct DepCell<T> {
    pub marker: Cell<i64>,
    pub cell: RefCell<T>
}

impl<T> DepCell<T> {
    pub fn new(value: T) -> DepCell<T> {
        DepCell {marker: Cell::new(-1), cell: RefCell::new(value) }
    }
}

impl<T> Deref for DepCell<T> {
    type Target = RefCell<T>;
    fn deref(&self) -> &RefCell<T> {
        &self.cell
    }
}

fn group_matches_arch(conn: &Connection, group_id: i64, arches: &Vec<String>) -> bool {
    match get_groups_kv_group_id(conn, group_id) {
        Ok(kvs) => { for kv in kvs {
                         if kv.key_value == "arch" {
                             if kv.val_value == "noarch" || arches.contains(&kv.val_value) {
                                 return true
                             } else {
                                 return false
                             }
                         }
                     }

                     return false
                   },
        Err(_) => return false
    }
}

// Given a requirement, find a list of groups providing it and return all of that as an expression
fn req_providers(conn: &Connection, arches: &Vec<String>, req: &Requirement, parents: &HashSet<i64>, cache: &mut HashMap<i64, Rc<DepCell<DepExpression>>>) -> Result<Option<Rc<DepCell<DepExpression>>>, String> {
    // helper function for converting a (Group, KeyVal) to Option<(group_id, Requirement)>
    fn provider_to_requirement(group: &Groups, kv: &KeyVal) -> Option<(i64, Requirement)> {
        let ext_val = match &kv.ext_value {
            &Some(ref ext_val) => ext_val,
            &None => return None
        };

        let requirement = match Requirement::from_str(ext_val.as_str()) {
            Ok(req) => req,
            Err(_)  => return None
        };

        Some((group.id, requirement))
    }

    // gather child requirements if necessary
    fn depclose_provider(conn: &Connection, arches: &Vec<String>, group_id: i64, parents: &HashSet<i64>, cache: &mut HashMap<i64, Rc<DepCell<DepExpression>>>) -> Result<Option<Rc<DepCell<DepExpression>>>, String> {
        if parents.contains(&group_id) {
            // This requirement is already satisfied, return
            Ok(None)
        } else {
            let provider_expr = try!(depclose_package(conn, arches, group_id, parents, cache));
            cache.insert(group_id, provider_expr.clone());
            Ok(Some(provider_expr))
        }
    }

    // flag to indicate that the requirement is satisfied via a parent of this expression, even if
    // the requirement list comes out empty
    let mut satisfied = false;

    let mut group_providers = match get_provider_groups(conn, req.name.as_str()) {
        Ok(providers) => {
            // We have a vector of (Groups, KeyVal) pairs, not all of which match the
            // version portion of the requirement expression. We want the matching
            // providers as DepExpression values, with any unsolvable providers removed
            let providers_checked = providers.iter()
                                             // convert the provides expression to a Requirement and return a (group_id, Requirement) tuple
                                             .filter_map(|&(ref group, ref kv)| provider_to_requirement(group, kv))
                                             // filter out any that don't match version-wise
                                             .filter(|&(_, ref provider_req)| provider_req.satisfies(&req))
                                             // filter out any that don't match arch-wise
                                             .filter(|&(ref group_id, _)| group_matches_arch(conn, *group_id, arches))
                                             // map the remaining providers to an expression, recursing to fetch the provider's requirements
                                             // any recursions that return Err unsatisfiable, so filter those out
                                             .filter_map(|(group_id, _)| match depclose_provider(conn, arches, group_id, parents, cache) {
                                                 Ok(provider) => {
                                                     // mark the requirement as satisfied
                                                     satisfied = true;
                                                     provider
                                                 },
                                                 Err(_) => None
                                             })
                                             .collect::<Vec<Rc<DepCell<DepExpression>>>>();

            providers_checked
        },
        Err(e) => return Err(e.to_string())
    };

    // If the requirement looks like a filename, check for groups providing the file *in addition to* rpm-provide
    if req.name.starts_with('/') {
        let mut file_providers = match get_groups_filename(conn, req.name.as_str()) {
            Ok(groups) => {
                // Unlike group_providers, there are no versions to care about here
                groups.iter().filter_map(|ref group| match depclose_provider(conn, arches, group.id, parents, cache) {
                    Ok(provider) => {
                        satisfied = true;
                        provider
                    },
                    Err(_) => None
                }).collect()
            },
            Err(e) => return Err(e.to_string())
        };
        group_providers.append(&mut file_providers);
    }

    if group_providers.is_empty() && !satisfied {
        // If there are no providers for the requirement, the requirement is unsatisfied, and that's an error
        Err(format!("Unable to satisfy requirement {}", req))
    } else if group_providers.is_empty() {
        // Requirement satisfied through a parent, but nothing new to add
        Ok(None)
    } else if group_providers.len() == 1 {
        // Only one provider, return it
        Ok(Some(group_providers[0].clone()))
    } else {
        // a choice among more than one provider
        Ok(Some(Rc::new(DepCell::new(DepExpression::Or(group_providers)))))
    }
}

// The expression for a package and its dependencies is:
// PACKAGE_group_id AND (PACKAGE_provides_1 AND PACKAGE_provides_2 AND ...) AND
//                      (PACKAGE_requires_1 AND PACKAGE_requires_2 AND ...) AND
//                      (NOT PACKAGE_obsoletes_1 AND NOT PACKAGE_obsoletes_2 AND ...) AND
//                      (NOT PACKAGE_conflicts_1 AND NOT PACKAGE_conflicts_2 AND ...)
//
// for each requires, this expands to a list of packages that provide the given requires expression
//   PACKAGE_requires_1 AND (PACKAGE_requires_1_provided_by_1 OR PACKAGES_requires_1_provided_by_2 OR ...)
//
// Each of the requires_provided_by atoms is a group id with a provides that satisfies the
// given requires. For each of these group ids, if the group has already been closed over in a
// parent of this expression, it's done. This check needs to be only on the parents, since the
// group id could exist in another branch of an OR. The child requirements for the group need to be
// in both branches, in case one of them gets eliminated during the solving step.
//
// Otherwise, recurse on the group and the requires_provided_by_atom expands to:
//
//    PACKAGE_requires_1_provided_by_1 AND (required_package_provides_1 AND required_package_provides_2 AND ...) ...
//
// Obsoletes are special in that the expression matches the package name, not a provider name. To
// handle this, in the provider list add a special Requirement of the form {name:"NAME: <name>", expr:Some(EqualTo, <version>)}
// and do the same thing to the name when processing Obsoletes.
//
// Obsoletes and conflicts do not need to be further expanded. Any conflicting packages that
// were closed over will be eliminated (or determined to be unresolvable) during depsolve.
//
// The end result is a boolean expression containing a mix of Requirements and group ids. The final
// result of depsolving will be a list of group ids. Each of the group id atoms is AND'd with its
// requirements so that during unit propagation a group id can only be removed from the expression
// if everything it needs can be removed from the expression, so that a group id is effectively the
// thing that can be turned on or off during solving.

fn depclose_package(conn: &Connection, arches: &Vec<String>, group_id: i64, parent_groups: &HashSet<i64>, cache: &mut HashMap<i64, Rc<DepCell<DepExpression>>>) -> Result<Rc<DepCell<DepExpression>>, String> {
    // If this value is cached, return it
    if let Some(expr) = cache.get(&group_id) {
        return Ok(expr.clone());
    }

    // TODO a functional hashmap or something similar would be super handy here
    // add this group to the parent groups, so that a cycle doesn't try to recurse on this group again
    let mut parent_groups_copy = parent_groups.clone();
    parent_groups_copy.insert(group_id);

    // Get all of the key/val based data we need
    // TODO would be nice to have a function or change this one to specify a key or keys, so we're not
    // getting all key/val data
    let (group_provides, group_obsoletes, group_conflicts) = match get_groups_kv_group_id(conn, group_id) {
        Ok(group_key_vals) => {
            // map a key/value pair into a Requirement
            fn kv_to_expr(kv: &KeyVal) -> Result<Rc<DepCell<DepExpression>>, String> {
                match &kv.ext_value {
                    &Some(ref ext_value) => Ok(Rc::new(DepCell::new(DepExpression::Atom(DepAtom::Requirement(try!(Requirement::from_str(ext_value.as_str()))))))),
                    &None                => Err(String::from("ext_value is not set"))
                }
            }

            fn kv_to_not_expr(kv: &KeyVal) -> Result<Rc<DepCell<DepExpression>>, String> {
                Ok(Rc::new(DepCell::new(DepExpression::Not(try!(kv_to_expr(kv))))))
            }

            let mut group_provides = Vec::new();
            let mut group_obsoletes = Vec::new();
            let mut group_conflicts = Vec::new();
            let mut name = None;
            let mut version = None;

            for kv in group_key_vals.iter() {
                match kv.key_value.as_str() {
                    "rpm-provide" => group_provides.push(kv_to_expr(kv)),
                    "rpm-conflict" => group_conflicts.push(kv_to_not_expr(kv)),
                    // obsolete matches the package name, not a provides, so handle it differently
                    "rpm-obsolete" => match &kv.ext_value {
                        &Some(ref ext_value) => match Requirement::from_str(ext_value.as_str()) {
                            Ok(base_req) => {
                                let new_req = Requirement{name: "PKG: ".to_string() + base_req.name.as_str(),
                                                          expr: base_req.expr};
                                group_obsoletes.push(Ok(Rc::new(DepCell::new(DepExpression::Not(Rc::new(DepCell::new(DepExpression::Atom(DepAtom::Requirement(new_req)))))))));
                            },
                            Err(e) => group_obsoletes.push(Err(e))
                        },
                        &None => group_obsoletes.push(Err("ext_value is not set".to_string()))
                    },
                    "name"         => name = Some(&kv.val_value),
                    "version"      => version = Some(&kv.val_value),
                    _ => {}
                }
            }

            match (name, version) {
                (Some(name), Some(version)) => match EVR::from_str(version.as_str()) {
                    Ok(evr) => { 
                        let req = Requirement{name: "PKG: ".to_string() + name.as_str(),
                                              expr: Some((ReqOperator::EqualTo, evr))};
                        group_provides.push(Ok(Rc::new(DepCell::new(DepExpression::Atom(DepAtom::Requirement(req))))));
                    },
                    Err(e)  => group_provides.push(Err(e))
                },
                _ => ()
            }

            // Collect the Vec<Result<Expression, String>>s into a Result<Vec<Expression>, String>
            let group_provides_result: Result<Vec<Rc<DepCell<DepExpression>>>, String> = group_provides.into_iter().collect();
            let group_obsoletes_result: Result<Vec<Rc<DepCell<DepExpression>>>, String> = group_obsoletes.into_iter().collect();
            let group_conflicts_result: Result<Vec<Rc<DepCell<DepExpression>>>, String> = group_conflicts.into_iter().collect();

            // unwrap the result or return the error
            (try!(group_provides_result), try!(group_obsoletes_result), try!(group_conflicts_result))
        },
        Err(e) => return Err(e.to_string())
    };

    // Collect the requirements
    let group_requirements = match get_requirements_group_id(conn, group_id) {
        Ok(requirements) => {
            // Map the data from the Requirements table into a rpm Requirement
            let gr_reqs: Vec<Requirement> = try!(requirements.iter().map(|r| Requirement::from_str(r.req_expr.as_str())).collect());

            // for each requirement, create an expression of (requirement AND requirement_providers)
            let mut group_requirements: Vec<Rc<DepCell<DepExpression>>> = Vec::new();
            for r in gr_reqs.iter() {
                // If only one group comes back as the requirement (i.e., there is only one
                // provider for the requirement), that group can be skipped in additional
                // requirements.
                // For instance, if our expression so far is something like:
                //    (req_1 AND (groupid=47 AND group_47_reqs)) AND (req_2 ...)
                // 
                // and req_2 is also satisified by groupid=47, we don't need another copy of
                // groupid=47 and its requirements.
                //
                // This isn't perfect, since there can still be extra copies depending on the order
                // things are processed in, but it should cut way down on extra copies of
                // everything.
                let providers = try!(req_providers(conn, arches, r, &parent_groups_copy, cache));
                let req_expr  = Rc::new(DepCell::new(DepExpression::Atom(DepAtom::Requirement(r.clone()))));
                match providers {
                    Some(provider_exp) => {
                        if Rc::new(DepCell::new(DepExpression::Atom(DepAtom::GroupId(group_id)))) == provider_exp {
                            parent_groups_copy.insert(group_id);
                        }
                        group_requirements.push(Rc::new(DepCell::new(DepExpression::And(vec![req_expr, provider_exp]))));
                    },
                    None => ()
                };
            }
            group_requirements
        },
        Err(e) => return Err(e.to_string())
    };

    let mut and_list = Vec::new();
    and_list.push(Rc::new(DepCell::new(DepExpression::Atom(DepAtom::GroupId(group_id)))));
    if !group_provides.is_empty() {
        and_list.push(Rc::new(DepCell::new(DepExpression::And(group_provides))));
    }

    if !group_requirements.is_empty() {
        and_list.push(Rc::new(DepCell::new(DepExpression::And(group_requirements))));
    }

    if !group_obsoletes.is_empty() {
        and_list.push(Rc::new(DepCell::new(DepExpression::And(group_obsoletes))));
    }

    if !group_conflicts.is_empty() {
        and_list.push(Rc::new(DepCell::new(DepExpression::And(group_conflicts))));
    }

    Ok(Rc::new(DepCell::new(DepExpression::And(and_list))))
}

pub fn close_dependencies(conn: &Connection, arches: &Vec<String>, packages: &Vec<String>) -> Result<DepExpression, String> {
    let mut req_list = Vec::new();
    let mut cache = HashMap::new();

    for p in packages.iter() {
        // Get all the groups with the given name, and then filter out all those with an invalid
        // architecture.  This will really only matter when we are called with a library package,
        // which could have been built for several arches.  Binary packages are typically single
        // arch.
        let mut group_list = Vec::new();
        match get_groups_name(conn, p, 0, -1) {
            Ok(groups) => { for grp in groups {
                                if group_matches_arch(conn, grp.id, arches) {
                                    group_list.push(try!(depclose_package(conn, arches, grp.id, &HashSet::new(), &mut cache)));
                                }
                            }
                          },
            Err(e)     => return Err(e.to_string())
        }
        req_list.push(Rc::new(DepCell::new(DepExpression::Or(group_list))));
    }

    Ok(DepExpression::And(req_list))
}
