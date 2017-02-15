//! BDCS API Server handlers
//!
//! # Overview
//!
//! The API server uses the [Rocket.rs](http://rocket.rs) web framework to handle requests.  The
//! handler functions are called by Rocket to handle requests for specific mountpoints.
//!
//! The [bdcs::db](bdcs/db/index.html) module is used for the database operations. None of the
//! handlers should be executing SQL on the database directly.
//!
//! Requests are via HTTP for now, eventually it will be https only.
//!
//! # Methods
//!
//! * `GET` - is used to retrieve results from the server. No changes are made to the state of the
//!   server using a `GET` request.
//! * `POST` - is used to initiate a change or an action on the server. eg. write a Recipe, start a
//!   compose, etc.
//!
//! All other HTTP methods are unused at this time.
//!
//! # HTTP Status Codes
//!
//! Status codes will be used along with JSON responses to indicate the success or failure of
//! requests.
//!
//! * `2xx` - Success, JSON response depends on the resource being accessed.
//! * `4xx` - Request failure, additional details in the JSON error response.
//! * `5xx` - Server errors, additional details in the JSON error response.
//!
//! # Versioning
//!
//! API access is always versioned, and old versions will remain accessible unless explicitly
//! deprecated for 1 version release cycle.
//!
//! The base path of the URLs are `/api/v0/` with the REST resource root starting after that.
//!
//! # REST
//!
//! URLs are used to describe the resources being accessed. Generally trying to follow the advice
//! [found here](http://blog.mwaysolutions.com/2014/06/05/10-best-practices-for-better-restful-api/)
//! except for point 6, HATEOAS.
//!
//! * Use plural nouns for resources, and HTTP Methods as the verbs.
//! * GET does not alter the state of the server
//! * Use sub-resources for relations (eg. TODO Add an example)
//!
//! ## Optional filter parameters
//!
//! Some API routes that can return large amounts of data, like `/projects/list`, default to returning
//! a limited number of results (currently the default is 20). The caller can change this limit by using
//! the `limit` parameter to set a different limit. eg. `/api/v0/recipes/list?limit=50`
//!
//! These responses can also be paginated by passing an `offset` parameter to skip some number of results
//! before returning values. eg. to get the 2nd 'page' of projects you would request
//! `/api/v0/recipes/list?offset=20`
//!
//! These can also be combined. eg. `/api/v0/recipes/list?limit=10&offset=40` to implement different page
//! lengths.
//!
//! # Responses
//!
//! All responses will be JSON objects. Responses to GET requests will have the response included
//! under a key set to the resource root. eg. `GET /api/v0/recipes/list` will return the list as
//! `{"recipes":[{"name":value, ...}, ...]}`
//!
//! Responses may also include extra metadata in other keys. eg. limit, offset, and total for
//! pagination results.
//!
//! ## Error Responses
//!
//! In addition to the HTTP Error codes, extra information will be included in a JSON response object with
//! `{"id": "internal error id", "msg": "Human readable message, suitable for passing to users"}`
//!
//! # Authentication
//!
//! This is still TBD.
//!
//! ## Authorization: Bearer tokens
//!
//! ## Basic Auth tokens
//!

// Copyright (C) 2016-2017 Red Hat, Inc.
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

use hyper::method::Method;
use rocket::http::hyper::header;
use rocket::response::{self, Responder, Response};


pub mod v0;
pub mod mock;
pub mod docs;

// defaults for queries that return multiple responses
pub static OFFSET: i64 = 0;
pub static LIMIT: i64 = 20;


/// This is used for optional query parameters that filter the results
///
/// Pass it to the handler as `filter: Filter` and it will (or won't) contain the offset and limit
/// arguments passed to the request.
///
#[derive(Debug, Serialize, FromForm)]
pub struct Filter {
    pub offset: Option<i64>,
    pub limit: Option<i64>
}

/// Response wrapper that adds CORS headers to the response
///
/// Based on the JSON response from Rocket's contrib library.
#[derive(Debug)]
pub struct CORS<R>(pub R);

impl<'r, R: Responder<'r>> CORS<R> {
    /// Return the wrapped type
    pub fn unwrap(self) -> R {
        self.0
    }
}

impl<'r, R: Responder<'r>> Responder<'r> for CORS<R> {
    fn respond(self) -> response::Result<'r> {
        Response::build_from(try!(self.0.respond()))
            .header(header::AccessControlAllowOrigin::Any)
            .header(header::AccessControlAllowMethods(vec![
                Method::Get, Method::Post, Method::Options]))
            .header(header::AccessControlAllowHeaders(vec![
                // Hyper uses the `unicase::Unicase` type to ensure comparisons are done
                // case-insensitively. Here, we use `into()` to convert to one from a `&str`
                // so that we don't have to import the type ourselves.
                "Origin".into(),
                "X-Requested-With".into(),
                "Content-Type".into(),
                "Accept".into(),
            ]))
            .ok()
    }
}
