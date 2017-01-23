//! BDCS Mock API handling
//!
// Copyright (C) 2017
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
//! # Overview
//!
//! This module provides a mock API service that reads json files from a static directory and serves up
//! the data from the file as-is.
//!
//! Requests like /route are read from a file named route.json
//! Requests like /route/param are read from route-param.json
//! Requests like /route/action/param are handled slightly differently, depending on the file.
//!
//! If the route-action.json file has an Array in it then it will be searched for a "name" key
//! matching param. If that is found just that matching entry will be returned.
//!
//! The mock files should follow the (API rules here)[path/to/rules.html], using a top level
//! object named the same as the route to hold the child object or array. Examples of valid
//! files can be found in the tests/results/ directory.
//!
//! # Example Response
//!
//! eg. /recipes/info/http-server
//! ```json
//! {"limit":20,"offset":0,"recipes":{"description":"An example http
//! server","modules":[{"name":"fm-httpd","version":"23.*"},{"name":"fm-php","version":"11.6.*"}],"name":"http-server","packages":[{"name":"tmux","version":"2.2"}]}}
//! ```
//!
use std::fs::File;
use std::path::Path;

use rocket::config;
use rocket_contrib::JSON;
use rocket::response::NamedFile;
use serde_json::{self, Value};

use api::{CORS, Filter, OFFSET, LIMIT};


/// A nice little macro to create simple Maps. Really convenient for
/// returning ad-hoc JSON messages.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate rocket_contrib;
/// use std::collections::HashMap;
/// # fn main() {
/// let map: HashMap<&str, usize> = map! {
///     "status" => 0,
///     "count" => 100
/// };
///
/// assert_eq!(map.len(), 2);
/// assert_eq!(map.get("status"), Some(&0));
/// assert_eq!(map.get("count"), Some(&100));
/// # }
/// ```
///
/// Borrowed from rocket.rs and modified to use serde_json::value::Map
macro_rules! json_map {
    ($($key:expr => $value:expr),+) => ({
        use serde_json::value::Map;
        let mut map = Map::new();
        $(map.insert($key.to_string(), $value);)+
        map
    });

    ($($key:expr => $value:expr),+,) => {
        json_map!($($key => $value),+)
    };
}


/// Handler for a bare route with offset and/or limit
#[get("/<route>?<filter>")]
pub fn static_route_filter(route: &str, filter: Filter) -> CORS<Option<NamedFile>> {
    static_json(route, None, filter.offset.unwrap_or(OFFSET), filter.limit.unwrap_or(LIMIT))
}

/// Handler for a bare route with no filtering
#[get("/<route>", rank=2)]
pub fn static_route(route: &str) -> CORS<Option<NamedFile>> {
    static_json(route, None, OFFSET, LIMIT)
}

/// Handler for a route with a single parameter and offset and/or limit
#[get("/<route>/<param>?<filter>")]
pub fn static_route_param_filter(route: &str, param: &str, filter: Filter) -> CORS<Option<NamedFile>> {
    static_json(route, Some(param), filter.offset.unwrap_or(OFFSET), filter.limit.unwrap_or(LIMIT))
}

/// Handler for a route with a single parameter and no filtering
#[get("/<route>/<param>", rank=2)]
pub fn static_route_param(route: &str, param: &str) -> CORS<Option<NamedFile>> {
    static_json(route, Some(param), OFFSET, LIMIT)
}

/// Handler for a route with an action, a parameter and offset/limit
#[get("/<route>/<action>/<param>?<filter>")]
pub fn static_route_action_filter(route: &str, action: &str, param: &str, filter: Filter) -> CORS<JSON<Value>> {
    filter_json(route, action, param, filter.offset.unwrap_or(OFFSET), filter.limit.unwrap_or(LIMIT))
}

/// Handler for a route with an action, a parameter and no filtering
#[get("/<route>/<action>/<param>", rank=2)]
pub fn static_route_action(route: &str, action: &str, param: &str) -> CORS<JSON<Value>> {
    filter_json(route, action, param, OFFSET, LIMIT)
}

/// Return a static json file based on the route and parameter passed to the API
///
/// # Arguments
///
/// * `route` - The 1st argument on the API path
/// * `param` - The 2nd argument on the API path
/// * `offset` - Number of results to skip before returning results. Default is 0.
/// * `limit` - Maximum number of results to return. It may return less. Default is 20.
///
/// # Response
///
/// * JSON response read from the file.
///
/// The filename returned is constructed by joining the route and the param with a '-' and
/// appending .json to it. eg. `/modules/list` will read from a file named `modules-list.json`
///
/// The filtering arguments, offset and limit, are ignored. Any offset or limit in the file
/// are returned as-is.
///
fn static_json(route: &str, param: Option<&str>, offset: i64, limit: i64) -> CORS<Option<NamedFile>> {
    info!("mock request"; "route" => route, "param" => param, "offset" => offset, "limit" => limit);

    let mock_path = config::active()
                           .unwrap()
                           .get_str("mockfiles_path")
                           .unwrap_or("/var/tmp/bdcs-mockfiles/");
    // TODO Better way to construct this...
    let param = match param {
        Some(p) => format!("-{}", p),
        None => "".to_string()
    };
    let file = format!("{}{}.json", route, param);
    CORS(NamedFile::open(Path::new(mock_path).join(file)).ok())
}

/// Filter the contents of a static json file based on the action and parameter passed to the API
///
/// # Arguments
///
/// * `route` - The 1st argument on the API path
/// * `action` - The 2nd argument on the API path
/// * `param` - The 3rd argument on the API path
/// * `offset` - Number of results to skip before returning results. Default is 0.
/// * `limit` - Maximum number of results to return. It may return less. Default is 20.
///
/// # Response
///
/// * JSON response selected from the file.
///
/// The filename to read is constructed by joining the route and the action with a '-' and
/// appending .json to it. eg. `/modules/info/http-server` will read from a file named `modules-info.json`
///
/// The filtering arguments, offset and limit, are ignored but are returned as part of the
/// response.
///
/// The file is expected to contain valid json with an object named for the route. If the route
/// object is not an array, it is returned as-is, along with the offset and limit values.
///
/// eg. A file with only an object:
/// ```json
/// {"modules":{"name": "http-server", ...}}
/// ```
///
/// If the route object contains an array then it is searched for a `"name"` key that is equal to the
/// `param` passed to the API. A response is constructed using the route and the single
/// object from the file. If nothing is found then a `null` is returned.
///
/// eg. A file with an array
/// ```json
/// {"modules":[{"name": "http-server", ...}, {"name": "nfs-server", ...}]}
/// ```
///
fn filter_json(route: &str, action: &str, param: &str, offset: i64, limit: i64) -> CORS<JSON<Value>> {
    info!("mock request"; "route" => route, "action" => action, "param" => param, "offset" => offset, "limit" => limit);
    let mock_path = config::active()
                           .unwrap()
                           .get_str("mockfiles_path")
                           .unwrap_or("/var/tmp/bdcs-mockfiles/");

    let file = format!("{}-{}.json", route, action);
    let json_data: Value = serde_json::from_reader(File::open(Path::new(mock_path).join(file))
                                                     .unwrap())
                              .unwrap();
    let json_obj = json_data.as_object().unwrap();

    debug!("json object"; "json" => format!("{:?}", json_obj));

    // Properly formatted with {"<route>": ...
    if let Some(api_route) = json_obj.get(route) {
        // If the route is an array search for a "name" key set to the value of param.
        if api_route.is_array() {
            if let Some(json_array) = api_route.as_array() {
                for item in json_array {
                    if item.find("name").unwrap_or(&Value::String("".to_string())) == &Value::String(param.to_string()) {
                        info!("Found it!"; "json" => format!("{:?}", item));
                        return CORS(JSON(Value::Object(json_map! { route => Value::Array(vec![item.clone()]),
                                    "offset" => Value::I64(offset),
                                    "limit" => Value::I64(limit) })));
                    }
                }
            }
        } else {
            // Not an array, just return it
            return CORS(JSON(Value::Object(json_map! { route => api_route.clone(),
                        "offset" => Value::I64(offset),
                        "limit" => Value::I64(limit) })));
        }
    }
    // Nothing to return
    CORS(JSON(Value::Object(json_map! { route => Value::Null,
              "offset" => Value::I64(offset),
              "limit" => Value::I64(limit) })))
}
