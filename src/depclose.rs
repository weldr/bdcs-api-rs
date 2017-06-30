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

pub type GroupId = i64;

/// A dependency expression
#[derive(Debug, Clone)]
pub enum DepExpression {
    Atom(GroupId),
    Not(GroupId),
    And(Vec<DepExpression>),
    Or(Vec<DepExpression>)
}

impl fmt::Display for DepExpression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DepExpression::Atom(ref id) =>  write!(f, "{}", id),
            DepExpression::And(ref lst) =>  { let strs: String = lst.iter().map(|x| x.to_string()).intersperse(" AND ".to_string()).collect();
                                              write!(f, "({})", strs)
                                            },
            DepExpression::Or(ref lst) =>   { let strs: String = lst.iter().map(|x| x.to_string()).intersperse(" OR ".to_string()).collect();
                                              write!(f, "({})", strs)
                                            },
            DepExpression::Not(ref id) => write!(f, "NOT {}", id)
        }
    }
}

fn group_matches_arch(conn: &Connection, group_id: i64, arches: &[String]) -> bool {
    match get_groups_kv_group_id(conn, group_id) {
        Ok(kvs) => { for kv in kvs {
                         if kv.key_value == "TextKey \"arch\"" {
                             return kv.val_value == Some("noarch".to_string()) || arches.contains(&kv.val_value.unwrap_or_default());
                         }
                     }

                     false
                   },
        Err(_) => false
    }
}

// Helper function for convert a (Group, KeyVal) provider pair to a requirement
fn provider_to_requirement(group: &Groups, kv: &KeyVal) -> Result<(GroupId, Requirement), String> {
    let ext_val = match kv.ext_value {
        Some(ref ext_val) => ext_val,
        None => return Err("ext_value not set".to_string())
    };
    Ok((group.id, Requirement::from(ext_val.as_str())))
}

// Helper function to convert a GroupId into a name = EVR Requirement
fn group_id_to_requirement(conn: &Connection, group_id: GroupId) -> Result<(GroupId, Requirement), String> {
    let mut name = None;
    let mut epoch = None;
    let mut version = None;
    let mut release = None;

    let group_key_vals_result = get_groups_kv_group_id(conn, group_id);
    match group_key_vals_result {
        Ok(group_key_vals) => {
            for kv in group_key_vals {
                match kv.key_value.as_str() {
                    "TextKey \"name\""      => name     = kv.val_value,
                    "TextKey \"version\""   => version  = kv.val_value,
                    "TextKey \"epoch\""     => epoch    = kv.val_value,
                    "TextKey \"release\""   => release  = kv.val_value,
                    _                       => ()
                }
            }
        },
        Err(e) => return Err(e.to_string())
    }

    // Everything is required except epoch
    let name_val = try!(name.ok_or("No name set"));
    let version_val = try!(version.ok_or("No version set"));
    let release_val = try!(release.ok_or("No release set"));

    // Epoch, if set, has to be parseable as a u32
    let epoch_val = match epoch {
        Some(e) => Some(try!(u32::from_str(e.as_str()).map_err(|_| "Unable to parse epoch"))),
        None    => None
    };

    Ok((group_id,
        Requirement{name: name_val, expr: Some((ReqOperator::EqualTo, EVR{epoch: epoch_val, version: version_val, release: release_val}))})
       )
}

// Given an obsolete, return a list of matching groups ids
// Unlike requires and conflicts, this matches against RPM name instead of rpm-provide
fn req_obsolete_ids(conn: &Connection, arches: &[String], req: &Requirement) -> Result<Vec<GroupId>, String> {
    let obsolete_providers = match get_groups_by_name(conn, req.name.as_str(), "rpm") {
        Ok(group_ids) => {
            // For each group, create a <name> = [epoch:]<version>-<release> Requirement
            let group_requirements: Vec<(GroupId, Requirement)> = try!(group_ids
                .iter()
                // Filter out the ones that don't match by arch
                .filter(|&id| group_matches_arch(conn, *id, arches))
                // Map the ids to Result<(GroupId, Requirement), String> and take care of the error case
                .map(|id| group_id_to_requirement(conn, *id))
                .collect());

            group_requirements
                .into_iter()
                // Filter out the ones that don't match by arch or version
                .filter(|&(_, ref conflict_req)| req.satisfies(conflict_req))
                // Pull out just the group ids
                .map(|(ref group_id, _)| *group_id)
                .collect()
        },
        Err(e) => return Err(e.to_string())
    };

    Ok(obsolete_providers)
}

// Given a requirement, return a list of groups that satisfy the requirement
fn req_provider_ids(conn: &Connection, arches: &[String], req: &Requirement) -> Result<Vec<GroupId>, String> {
    let mut group_providers = Vec::new();

    // Find matches in the rpm-provide data
    match get_provider_groups(conn, req.name.as_str()) {
        Ok(providers) => {
            // We have a vector of (Groups, KeyVal) pairs, not all of which match the
            // version portion of the requirement expression.
            let providers_checked = providers.iter()
                                    // convert the pair of db records to (GroupId, Requirement), removing any providers that can't be parsed
                                    .filter_map(|&(ref group, ref kv)| provider_to_requirement(group, kv).ok())
                                    // filter out any that don't match version-wise
                                    .filter(|&(_, ref provider_req)| provider_req.satisfies(req))
                                    // filter out any that don't match arch-wise
                                    .filter(|&(ref group_id, _)| group_matches_arch(conn, *group_id, arches))
                                    // and pull out just the remaining group ids
                                    .map(|(ref group_id, _)| *group_id);

            group_providers.extend(providers_checked);
        },
        Err(e) => return Err(e.to_string())
    }

    // If the requirement looks like a filename, check for groups providing the file *in addition to* rpm-provide
    if req.name.starts_with('/') {
        match get_groups_filename(conn, req.name.as_str()) {
            Ok(groups) => {
                // Unlike group_providers, there are no versions to care about here
                let providers_checked = groups.iter()
                                              // check if the arch matches
                                              .filter(|&group| group_matches_arch(conn, group.id, arches))
                                              // pull out just the id
                                              .map(|group| group.id);
                group_providers.extend(providers_checked);
            }
            Err(e) => return Err(e.to_string())
        }
    }

    // sort, dedup, and return
    group_providers.sort();
    group_providers.dedup();
    Ok(group_providers)
}

// depclose a single group id
// The expression for a package and its dependencies is:
// PACKAGE_group_id AND requirement_1 AND requirement_2 ...
//
// each requirement may be satisfied by more than one group_id, which is expressed as 
//
//    (requirement_1_provider_1 OR requirement_1_provider 2 ... )
//
// This function recurses on groups selected as requirements, stopping when it detects a cycle via
// the parents HashSet. So, supposing there are packages:
//
//   A Requires B
//   B Requires C
//   C Requires A
//
// Calling depclose on A will recurse on B with a parents of {A}, recurse on C with a parents of
// {A,B}, and then finish since A is already being depclosed in this expression.
fn depclose_package(conn: &Connection, arches: &[String], group_id: GroupId, parents: &HashSet<GroupId>, cache: &mut HashMap<GroupId, DepExpression>) -> Result<DepExpression, String> {
    fn kv_to_req(kv: &KeyVal) -> Result<Requirement, String> {
        match kv.ext_value {
            Some(ref ext_value) => Ok(Requirement::from(ext_value.as_str())),
            None => Err("ext_value is not set".to_string())
        }
    }

    let mut group_requirements: Vec<DepExpression> = Vec::new();

    // If this value is cached, return it
    if let Some(expr) = cache.get(&group_id) {
        return Ok(expr.clone())
    }

    // TODO a functional hashap or something similar would be super handy here
    // add this group to the parent groups, so that a cycle doesn't try to recurse on this group again
    let mut parent_groups_copy = parents.clone();
    parent_groups_copy.insert(group_id);

    // Get all of the key/val based data we need
    // TODO maybe make a new version of get_groups_kv_group_id that takes the key(s?) as an argument
    let mut conflicts = Vec::new();
    let mut obsoletes = Vec::new();
    match get_groups_kv_group_id(conn, group_id) {
        Ok(ref group_key_vals) => {
            for kv in group_key_vals.iter() {
                match kv.key_value.as_str() {
                    "TextKey \"rpm-conflict\"" => conflicts.push(try!(kv_to_req(kv))),
                    "TextKey \"rpm-obsolete\"" => obsoletes.push(try!(kv_to_req(kv))),
                    _                          => ()
                }
            }
        },
        Err(e) => return Err(e.to_string())
    };

    // look for packages that provide the conflict expressions
    for c in &conflicts {
        let group_ids = try!(req_provider_ids(conn, arches, c));
        group_requirements.extend(group_ids.into_iter().map(DepExpression::Not));
    }

    // look for packages with names matching the obsolete expressions
    for o in &obsoletes {
        let group_ids = try!(req_obsolete_ids(conn, arches, o));
        group_requirements.extend(group_ids.into_iter().map(DepExpression::Not));
    }

    // Collect the requirements
    match get_requirements_group_id(conn, group_id) {
        Ok(requirements) => {
            // Map the data from the Requirements table into a rpm Requirement
            let gr_reqs: Vec<Requirement> = requirements.iter().map(|r| Requirement::from(r.req_expr.as_str())).collect();

            for r in gr_reqs {
                // Find the providers that satisfy the requirement
                let provider_ids = try!(req_provider_ids(conn, arches, &r));

                // If there are no providers, that's an error
                if provider_ids.is_empty() {
                    return Err(format!("Unable to satisfy requirement {}", r));
                }

                // If any of the provider ids have already been closed over, we're done with this
                // requirement (there is already a mandatory provider, so the requirement is satisfied)
                if provider_ids.iter().any(|id| parent_groups_copy.contains(id)) {
                    continue;
                }

                // if the providers are new, recurse over their requirements
                let mut req_providers: Vec<DepExpression> = Vec::new();
                for p in &provider_ids {
                    req_providers.push(try!(depclose_package(conn, arches, *p, &parent_groups_copy, cache)));
                }

                // If only one group comes back as the requirement (i.e., there is only one
                // provider for the requirement), that group can be skipped in additional
                // requirements.
                // For instance, if we have:
                //    Group1 Requires B
                //    Group1 Requires C
                //
                //    Group2 Provides B
                //    Group2 Provides C
                //    Group3 Provides C
                //
                // When processing the B requirement, we get just Group2 and its requirements. When
                // processing the C requirement, the requirement is satisfied by (Group2 OR
                // Group3). We can just skip that second requirement, since it's already satisfied
                // by Group2, which is already mandatory for this expression.  This is essentially
                // an early, crappy form of unit propagation.
                // 
                // This isn't perfect, since there can still be extra copies depending on the order
                // things are processed in, but it should cut way down on extra copies of
                // everything.
                if provider_ids.len() == 1 {
                    parent_groups_copy.insert(provider_ids[0]);
                }

                // the expression for this requirement is an OR of the possible providers
                if req_providers.len() == 1 {
                    group_requirements.push(req_providers.remove(0));
                } else {
                    group_requirements.push(DepExpression::Or(req_providers));
                }
            }
        },
        Err(e) => return Err(e.to_string())
    };

    // Add the package itself to the requirements
    group_requirements.push(DepExpression::Atom(group_id));

    // Cache the expression and return
    let expr = if group_requirements.len() == 1 {
        group_requirements.remove(0)
    } else {
        DepExpression::And(group_requirements)
    };
    cache.insert(group_id, expr.clone());

    Ok(expr)
}

/// Gathers all possible dependencies for a package.
///
/// # Arguments
///
/// * `conn` - The database connection
/// * `arches` - The package architectures to select. e.g., x86_64, i686
/// * `package` - The package names to select
///
/// # Returns
///
/// * On success an unsolved DepExpression, describing the selected packages and the packages they
///   require and conflict with.
///
/// * On error, a string describing the error. This could be because a package does not exist or
///   its dependencies cannot be found.
///
pub fn close_dependencies(conn: &Connection, arches: &[String], packages: &[String]) -> Result<DepExpression, String> {
    let mut req_list: Vec<DepExpression> = Vec::new();
    let mut cache: HashMap<GroupId, DepExpression> = HashMap::new();

    for p in packages {
        // Get all the groups with the given name, and then filter out all those with an invalid
        // architecture.  This will really only matter when we are called with a library package,
        // which could have been built for several arches.  Binary packages are typically single
        // arch.
        let mut group_list = Vec::new();
        match get_groups_name(conn, p, 0, -1) {
            Ok(groups) => { if groups.is_empty() {
                                return Err(format!("No package named {}", p));
                            }

                            for grp in groups {
                                if group_matches_arch(conn, grp.id, arches) {
                                    group_list.push(try!(depclose_package(conn, arches, grp.id, &HashSet::new(), &mut cache)));
                                }
                            }
                           },
            Err(e)     => return Err(e.to_string())
        }

        // if it's just one thing, don't wrap it
        if group_list.len() == 1 {
            req_list.push(group_list.remove(0));
        } else {
            req_list.push(DepExpression::Or(group_list));
        }
    }

    // Like above: don't wrap the result if it's only one thing
    if req_list.len() == 1 {
        Ok(req_list.remove(0))
    } else {
        Ok(DepExpression::And(req_list))
    }
}

// Test functions
// TODO share this between here and tests/db.rs
#[cfg(test)]
// clippy isn't very good at firguring out we use this in tests
#[cfg_attr(feature="cargo-clippy", allow(unused_macros))]
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

// provider_to_requirement: takes a group and key/val and returns a (GroupId, Requirement)
#[test]
fn test_provider_to_requirement_err() {
    assert!(provider_to_requirement(&Groups{id: 1, name: "whatever".to_string(), group_type: "test".to_string()},
                                    &KeyVal{id: 1, key_value: "TextKey \"rpm-provide\"".to_string(), val_value: Some("something".to_string()), ext_value: None}
                                   ).is_err());
}

#[test]
fn test_provider_to_requirement_ok() -> () {
    assert_eq!(provider_to_requirement(&Groups{id: 47, name: "whatever".to_string(), group_type: "test".to_string()},
                                       &KeyVal{id: 1, key_value: "TextKey \"rpm-provide".to_string(), val_value: Some("something".to_string()), ext_value: Some("something >= 1.0".to_string())}),
               Ok((47, Requirement::from("something >= 1.0"))));
}

// group_matches_arch: bool for whether a given group matches a list of arches
#[cfg(test)]
// clippy isn't very good at figuring out what is used in tests for some reason
#[cfg_attr(feature="cargo-clippy", allow(unused_imports))]
#[cfg_attr(feature="cargo-clippy", allow(dead_code))]
mod test_group_matches_arch {
    use depclose::*;
    use test_helper::*;
    use rusqlite::{self, Connection};

    fn test_data() -> rusqlite::Result<Connection> {
        create_test_packages(&[
                             testpkg("testy", None, "1.0", "1", "x86_64", &[], &[], &[], &[]),
                             testpkg("testy", None, "1.0", "1", "i686", &[], &[], &[], &[]),
                             testpkg("testy", None, "1.0", "1", "s390x", &[], &[], &[], &[]),
                             testpkg("testy", None, "1.0", "1", "noarch", &[], &[], &[], &[])
        ])
    }

    #[test]
    fn test_no_arch() -> () {
        let conn = test_data().unwrap();
        let x86_id = get_nevra_group_id(&conn, "testy", None, "1.0", "1", "x86_64");
        let i686_id = get_nevra_group_id(&conn, "testy", None, "1.0", "1", "i686");
        let noarch_id = get_nevra_group_id(&conn, "testy", None, "1.0", "1", "noarch");

        assert_eq!(group_matches_arch(&conn, x86_id, &vec![]), false);
        assert_eq!(group_matches_arch(&conn, i686_id, &vec![]), false);
        assert_eq!(group_matches_arch(&conn, noarch_id, &vec![]), true);
    }

    #[test]
    fn test_single_arch() -> () {
        let conn = test_data().unwrap();
        let x86_id = get_nevra_group_id(&conn, "testy", None, "1.0", "1", "x86_64");
        let i686_id = get_nevra_group_id(&conn, "testy", None, "1.0", "1", "i686");
        let noarch_id = get_nevra_group_id(&conn, "testy", None, "1.0", "1", "noarch");

        let x86_arches = vec!["x86_64".to_string()];
        let i686_arches = vec!["i686".to_string()];

        assert_eq!(group_matches_arch(&conn, x86_id, &x86_arches), true);
        assert_eq!(group_matches_arch(&conn, i686_id, &x86_arches), false);
        assert_eq!(group_matches_arch(&conn, noarch_id, &x86_arches), true);

        assert_eq!(group_matches_arch(&conn, x86_id, &i686_arches), false);
        assert_eq!(group_matches_arch(&conn, i686_id, &i686_arches), true);
        assert_eq!(group_matches_arch(&conn, noarch_id, &i686_arches), true);
    }

    #[test]
    fn test_multi_arch() -> () {
        let conn = test_data().unwrap();
        let x86_id = get_nevra_group_id(&conn, "testy", None, "1.0", "1", "x86_64");
        let i686_id = get_nevra_group_id(&conn, "testy", None, "1.0", "1", "i686");
        let s390x_id = get_nevra_group_id(&conn, "testy", None, "1.0", "1", "s390x");
        let noarch_id = get_nevra_group_id(&conn, "testy", None, "1.0", "1", "noarch");

        let multi_arches = vec!["x86_64".to_string(), "i686".to_string()];

        assert_eq!(group_matches_arch(&conn, x86_id, &multi_arches), true);
        assert_eq!(group_matches_arch(&conn, i686_id, &multi_arches), true);
        assert_eq!(group_matches_arch(&conn, s390x_id, &multi_arches), false);
        assert_eq!(group_matches_arch(&conn, noarch_id, &multi_arches), true);
    }
}

// group_id_to_requirement: convert a GroupId to a name-based requirement, for Obsoletes
#[cfg(test)]
#[cfg_attr(feature="cargo-clippy", allow(unused_imports))]
#[cfg_attr(feature="cargo-clippy", allow(dead_code))]
mod test_group_id_to_requirement {
    use depclose::*;
    use test_helper::*;
    use rusqlite::{self, Connection};

    fn test_data() -> rusqlite::Result<Connection> {
        create_test_packages(&[
                             testpkg("test-package-1", None, "1.0", "1", "x86_64",
                                     &["test-package-1 = 1.0-1"],
                                     &[],
                                     &[],
                                     &[]),
                             testpkg("test-package-2", Some(47), "9.5.2", "3", "x86_64",
                                     &["test-package-2 = 47:9.5.2-3"],
                                     &[],
                                     &[],
                                     &[])
        ])
    }

    #[test]
    fn test_1() {
        let conn = test_data().unwrap();
        let group_id = get_nevra_group_id(&conn, "test-package-1", None, "1.0", "1", "x86_64");
        assert_eq!(group_id_to_requirement(&conn, group_id),
                   Ok((group_id, Requirement::from("test-package-1 = 1.0-1"))));
    }

    #[test]
    fn test_2() {
        let conn = test_data().unwrap();
        let group_id = get_nevra_group_id(&conn, "test-package-2", Some(47), "9.5.2", "3", "x86_64");
        assert_eq!(group_id_to_requirement(&conn, group_id),
                   Ok((group_id, Requirement::from("test-package-2 = 47:9.5.2-3"))));
    }
}

// req_obsolete_ids: return a list of group ids that match an obsolete requirement
#[cfg(test)]
#[cfg_attr(feature="cargo-clippy", allow(unused_imports))]
#[cfg_attr(feature="cargo-clippy", allow(dead_code))]
mod test_req_obsolete_ids {
    use depclose::*;
    use test_helper::*;
    use rusqlite::{self, Connection};

    fn test_data() -> rusqlite::Result<Connection> {
        create_test_packages(&[
                             // Normal looking package, provides itself
                             testpkg("test-package-1", None, "1.0", "1", "x86_64",
                                     &["test-package-1 = 1.0-1"],
                                     &[],
                                     &[],
                                     &[]),
                             
                             // Provides does not match name, to ensure match is against name
                             testpkg("test-package-2", None, "1.0", "1", "x86_64",
                                     &["other-provides = 1.0"],
                                     &[],
                                     &[],
                                     &[])
         ])
    }


    #[test]
    fn test_empty() {
        let conn = test_data().unwrap();
        let arches = vec!["x86_64".to_string()];
        let test_req = Requirement::from("does-not-exist = 1.0-1");
        let test_result = vec![];
        let test_data = req_obsolete_ids(&conn, &arches, &test_req).unwrap();

        assert_eq_no_order!(test_data, test_result);
    }

    #[test]
    fn test_normal() {
        let conn = test_data().unwrap();
        let arches = vec!["x86_64".to_string()];
        let test_req = Requirement::from("test-package-1 = 1.0-1");

        let test_group_id = get_nevra_group_id(&conn, "test-package-1", None, "1.0", "1", "x86_64");
        let test_result = vec![test_group_id];
        let test_data = req_obsolete_ids(&conn, &arches, &test_req).unwrap();

        assert_eq_no_order!(test_data, test_result);
    }

    #[test]
    fn test_normal_version_match() {
        let conn = test_data().unwrap();
        let arches = vec!["x86_64".to_string()];
        let test_req = Requirement::from("test-package-1 >= 0.9");

        let test_group_id = get_nevra_group_id(&conn, "test-package-1", None, "1.0", "1", "x86_64");
        let test_result = vec![test_group_id];
        let test_data = req_obsolete_ids(&conn, &arches, &test_req).unwrap();

        assert_eq_no_order!(test_data, test_result);
    }

    #[test]
    fn test_normal_no_version_match() {
        let conn = test_data().unwrap();
        let arches = vec!["x86_64".to_string()];
        let test_req = Requirement::from("test-package-1 >= 1.1");

        let test_result = vec![];
        let test_data = req_obsolete_ids(&conn, &arches, &test_req).unwrap();

        assert_eq_no_order!(test_data, test_result);
    }

    #[test]
    fn test_name_only() {
        let conn = test_data().unwrap();
        let arches = vec!["x86_64".to_string()];
        let test_req = Requirement::from("test-package-2 = 1.0-1");

        let test_group_id = get_nevra_group_id(&conn, "test-package-2", None, "1.0", "1", "x86_64");
        let test_result = vec![test_group_id];
        let test_data = req_obsolete_ids(&conn, &arches, &test_req).unwrap();

        assert_eq_no_order!(test_data, test_result);
    }

    #[test]
    fn test_provide_only() {
        let conn = test_data().unwrap();
        let arches = vec!["x86_64".to_string()];
        let test_req = Requirement::from("other-provides = 1.0");

        let test_result = vec![];
        let test_data = req_obsolete_ids(&conn, &arches, &test_req).unwrap();

        assert_eq_no_order!(test_data, test_result);
    }
}

// req_providers_ids: return a list of GroupIds that provide a given requirement
// Like the obsoletes test, except:
//   - matches are against Provides data, not package name
//   - can also match filenames.
#[cfg_attr(feature="cargo-clippy", allow(unused_imports))]
#[cfg_attr(feature="cargo-clippy", allow(dead_code))]
#[cfg(test)]
mod test_req_provider_ids {
    use test_helper::*;
    use rusqlite::{self, Connection};
    use depclose::*;
    use std::collections::HashSet;

    #[cfg(test)]
    fn test_data() -> rusqlite::Result<Connection> {
        let test_data = try!(create_test_packages(&[
                             // Normal looking package, provides itself
                             testpkg("test-package-1", None, "1.0", "1", "x86_64",
                                     &["test-package-1 = 1.0-1"],
                                     &[],
                                     &[],
                                     &[]),
                             
                             // Provides does not match name, to ensure match is against name
                             testpkg("test-package-2", None, "1.0", "1", "x86_64",
                                     &["other-provides = 1.0"],
                                     &[],
                                     &[],
                                     &[]),

                             // Package with filename provides
                             testpkg("test-package-3", None, "1.0", "1", "x86_64",
                                     &["test-package-3 = 1.0-1",
                                       "/provided/file"],
                                     &[],
                                     &[],
                                     &[])
        ]));

        // Add another file to test-package-3
        let package_3_id = get_nevra_group_id(&test_data, "test-package-3", None, "1.0", "1", "x86_64");

        try!(test_data.execute_named("
            insert into files (path, file_user, file_group, mtime)
            values (:path, :file_user, :file_group, :mtime)",
            &[(":path", &"/actual/file".to_string()),
              (":file_user", &"root".to_string()),
              (":file_group", &"root".to_string()),
              (":mtime", &0)]));
        let file_id = test_data.last_insert_rowid();

        try!(test_data.execute_named("insert into group_files (group_id, file_id) values (:group_id, :file_id)",
            &[(":group_id", &package_3_id),
              (":file_id", &file_id)]));

        Ok(test_data)
    }

    #[test]
    fn test_empty() {
        let conn = test_data().unwrap();
        let arches = vec!["x86_64".to_string()];
        let test_req = Requirement::from("does-not-exist = 1.0-1");
        let test_result = vec![];
        let test_data = req_provider_ids(&conn, &arches, &test_req).unwrap();

        assert_eq_no_order!(test_data, test_result);
    }

    #[test]
    fn test_normal() {
        let conn = test_data().unwrap();
        let arches = vec!["x86_64".to_string()];
        let test_req = Requirement::from("test-package-1 = 1.0-1");

        let test_group_id = get_nevra_group_id(&conn, "test-package-1", None, "1.0", "1", "x86_64");
        let test_result = vec![test_group_id];
        let test_data = req_provider_ids(&conn, &arches, &test_req).unwrap();

        assert_eq_no_order!(test_data, test_result);
    }

    #[test]
    fn test_normal_version_match() {
        let conn = test_data().unwrap();
        let arches = vec!["x86_64".to_string()];
        let test_req = Requirement::from("test-package-1 >= 0.9");

        let test_group_id = get_nevra_group_id(&conn, "test-package-1", None, "1.0", "1", "x86_64");
        let test_result = vec![test_group_id];
        let test_data = req_provider_ids(&conn, &arches, &test_req).unwrap();

        assert_eq_no_order!(test_data, test_result);
    }

    #[test]
    fn test_normal_no_version_match() {
        let conn = test_data().unwrap();
        let arches = vec!["x86_64".to_string()];
        let test_req = Requirement::from("test-package-1 >= 1.1");

        let test_result = vec![];
        let test_data = req_provider_ids(&conn, &arches, &test_req).unwrap();

        assert_eq_no_order!(test_data, test_result);
    }

    #[test]
    fn test_name_only() {
        let conn = test_data().unwrap();
        let arches = vec!["x86_64".to_string()];
        let test_req = Requirement::from("test-package-2 = 1.0-1");

        let test_result = vec![];
        let test_data = req_provider_ids(&conn, &arches, &test_req).unwrap();

        assert_eq_no_order!(test_data, test_result);
    }

    #[test]
    fn test_provide_only() {
        let conn = test_data().unwrap();
        let arches = vec!["x86_64".to_string()];
        let test_req = Requirement::from("other-provides = 1.0");

        let test_group_id = get_nevra_group_id(&conn, "test-package-2", None, "1.0", "1", "x86_64");
        let test_result = vec![test_group_id];
        let test_data = req_provider_ids(&conn, &arches, &test_req).unwrap();

        assert_eq_no_order!(test_data, test_result);
    }

    #[test]
    fn test_path_normal() {
        let conn = test_data().unwrap();
        let arches = vec!["x86_64".to_string()];
        let test_req = Requirement::from("/actual/file");

        let test_group_id = get_nevra_group_id(&conn, "test-package-3", None, "1.0", "1", "x86_64");
        let test_result = vec![test_group_id];
        let test_data = req_provider_ids(&conn, &arches, &test_req).unwrap();

        assert_eq_no_order!(test_data, test_result);
    }

    #[test]
    fn test_path_provided() {
        let conn = test_data().unwrap();
        let arches = vec!["x86_64".to_string()];
        let test_req = Requirement::from("/provided/file");

        let test_group_id = get_nevra_group_id(&conn, "test-package-3", None, "1.0", "1", "x86_64");
        let test_result = vec![test_group_id];
        let test_data = req_provider_ids(&conn, &arches, &test_req).unwrap();

        assert_eq_no_order!(test_data, test_result);
    }

    #[test]
    fn test_path_empty() {
        let conn = test_data().unwrap();
        let arches = vec!["x86_64".to_string()];
        let test_req = Requirement::from("/nosuch/file");

        let test_result = vec![];
        let test_data = req_provider_ids(&conn, &arches, &test_req).unwrap();

        assert_eq_no_order!(test_data, test_result);
    }
}

#[cfg(test)]
#[cfg_attr(feature="cargo-clippy", allow(unused_imports))]
#[cfg_attr(feature="cargo-clippy", allow(dead_code))]
mod test_depclose_package {
    use depclose::*;
    use test_helper::*;
    use rusqlite::{self, Connection};

    fn test_data() -> rusqlite::Result<Connection> {
        create_test_packages(&[
            // A requires B
            testpkg("test-package-A", None, "1.0", "1", "x86_64",
                    &["test-package-A = 1.0-1"],
                    &["test-package-B"],
                    &[],
                    &[]),

            testpkg("test-package-B", None, "1.0", "1", "x86_64",
                    &["test-package-B = 1.0-1"],
                    &[],
                    &[],
                    &[])
        ])
    }

    // no-order depexpression compare
    pub fn exprcmp(e1: &DepExpression, e2: &DepExpression) -> bool {
        match (e1, e2) {
            (&DepExpression::Atom(id1), &DepExpression::Atom(id2)) |
            (&DepExpression::Not(id1),  &DepExpression::Not(id2)) => id1 == id2,
            (&DepExpression::And(ref v1), &DepExpression::And(ref v2))   =>
                v1.iter().all(|item1| v2.iter().any(|item2| exprcmp(item1, item2))),
            (&DepExpression::Or(ref v1),  &DepExpression::Or(ref v2))    =>
                v1.iter().all(|item1| v2.iter().any(|item2| exprcmp(item1, item2))),
            _ => false
        }
    }


    #[test]
    fn test_1() {
        let conn = test_data().unwrap();
        let arches = vec!["x86_64".to_string()];
        
        let group_id_a = get_nevra_group_id(&conn, "test-package-A", None, "1.0", "1", "x86_64");
        let group_id_b = get_nevra_group_id(&conn, "test-package-B", None, "1.0", "1", "x86_64");

        let test_result = DepExpression::And(vec![DepExpression::Atom(group_id_a), DepExpression::Atom(group_id_b)]);
        let test_data = depclose_package(&conn, &arches, group_id_a, &HashSet::new(), &mut HashMap::new()).unwrap();

        println!("expected: {:?}", test_result);
        println!("got: {:?}", test_data);
        assert!(exprcmp(&test_result, &test_data))
    }
}
