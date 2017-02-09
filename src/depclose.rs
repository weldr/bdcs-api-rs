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
use rusqlite::{self, Connection};
use std::collections::HashSet;
use std::fmt;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NEVRA {
    pub name: String,
    pub epoch: Option<String>,
    pub version: String,
    pub release: String,
    pub arch: String
}

impl fmt::Display for NEVRA {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.epoch {
            None            => write!(f, "{}-{}-{}.{}", self.name, self.version, self.release, self.arch),
            Some(ref epoch) => write!(f, "{}:{}-{}-{}.{}", epoch, self.name, self.version, self.release, self.arch)
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Proposition {
    Obsoletes(String, String),
    Provides(NEVRA, String),
    Requires(NEVRA, String),
}

fn get_nevra_group_id(conn: &Connection, id: i64) -> NEVRA {
    let kvs = get_groups_kv_group_id(conn, id);

    let mut name = String::from("");
    let mut epoch = None;
    let mut ver = String::from("");
    let mut rel = String::from("");
    let mut arch = String::from("");

    for row in kvs.unwrap() {
        match row.key_value.as_ref() {
            "name"      => name = row.val_value.clone(),
            "epoch"     => epoch = Some(row.val_value.clone()),
            "version"   => ver = row.val_value.clone(),
            "release"   => rel = row.val_value.clone(),
            "arch"      => arch = row.val_value.clone(),
            _           => continue,
        }
    }

    NEVRA { name: name, epoch: epoch, version: ver, release: rel, arch: arch }
}

fn find_provider_for_name(conn: &Connection, thing: &str) -> Vec<(i64, Proposition)> {
    let mut contents = Vec::new();

    match get_provider_groups(conn, thing) {
        Ok(providers)   => { for tup in providers {
                                 let nevra = get_nevra_group_id(conn, tup.0.id);
                                 contents.push((tup.0.id, Proposition::Provides (nevra, String::from(thing))));
                             }
                           }
    
        Err(_)          => { }
    }

    contents
}

fn find_group_containing_file(conn: &Connection, thing: &str) -> Vec<(i64, Proposition)> {
    let mut contents = Vec::new();

    match get_groups_filename(conn, thing) {
        Ok(providers)   => { for tup in providers {
                                 let nevra = get_nevra_group_id(conn, tup.id);
                                 contents.push((tup.id, Proposition::Provides (nevra, String::from(thing))));
                             }
                           }
        Err(_)          => { }
    }

    contents
}

fn what_obsoletes(conn: &Connection, id: i64) -> Vec<(String, String)> {
    let mut contents = Vec::new();

    match get_group_obsoletes(conn, id) {
        Ok(obsoleters)  => { for tup in obsoleters {
                                 contents.push((tup.0.name.clone(), tup.1.ext_value.clone().unwrap()));
                             }
                           }
        Err(_)          => { }
    }

    contents
}

pub fn close_dependencies(conn: &Connection, packages: Vec<String>) -> rusqlite::Result<Vec<Proposition>> {
    let mut props = HashSet::new();
    let mut seen = HashSet::new();
    let mut worklist = packages.clone();

    while !worklist.is_empty() {
        let hd = worklist.pop().unwrap();

        // We've seen this before, don't gather it up again.
        if seen.contains(&hd) {
            continue;
        }

        let mut providers = find_provider_for_name(conn, hd.as_str());

        // If the requirement looks like a filename, also look for packages
        // providing the file.
        if hd.starts_with('/') {
            let mut file_providers = find_group_containing_file(conn, hd.as_str());
            providers.append(&mut file_providers);
        }

        // Extract the group IDs from each provider tuple.
        let group_ids: Vec<i64> = providers.clone().into_iter().map(|x| x.0).collect();

        // Get the requirements and obsoletes for each.
        let mut reqs = Vec::new();
        let mut obs = Vec::new();

        for i in group_ids {
            match get_requirements_group_id(conn, i) {
                Ok(lst) => { let new = lst.into_iter().map(|x| x.req_expr).collect();
                             reqs.push(new);
                           }
                Err(_)  => { reqs.push(Vec::new()); }
            }

            let mut lst = what_obsoletes(conn, i);
            obs.append(&mut lst);
        }

        // Add the new propositions to the set.
        for i in &providers {
            props.insert(i.1.clone());
        }

        for i in &obs {
            props.insert(Proposition::Obsoletes (i.0.clone(), i.1.clone()));
        }

        if !providers.is_empty() {
            for i in 0..providers.len()-1 {
                for j in &reqs[i] {
                    let nevra = get_nevra_group_id(conn, providers[i].0);
                    props.insert(Proposition::Requires (nevra, j.clone()));
                }
            }
        }

        // Add the thing we just processed to the seen list so we don't look at it again.
        seen.insert(hd);

        // And then add all the requirements and obsoletes we just discovered to the
        // worklist and loop.  Remember reqs is a vector of vectors, so it has to be
        // flattened.
        let mut flattened : Vec<String> = reqs.into_iter().flat_map(|x| x.into_iter()).collect();
        worklist.append(&mut flattened);

        let mut obsolete_names = obs.into_iter().map(|x| x.0).collect();
        worklist.append(&mut obsolete_names);
    }

    Ok(props.into_iter().collect())
}
