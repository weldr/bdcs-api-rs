//! BDCS API Server handlers
//!
// Copyright (C) 2016
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
//
pub mod v0;


use config::BDCSConfig;
use hyper::header;
use nickel::{Request, Response, MiddlewareResult};

/// Enable CORS support
///
/// # Arguments
///
/// * `_req` - Unused Request structure
/// * `res` - Response to me modified
///
/// # Returns
///
/// * A `MiddlewareResult`
///
/// See [the Mozilla page](https://developer.mozilla.org/en-US/docs/Web/HTTP/Access_control_CORS)
/// for more details about CORS.
///
/// This modifies the headers so that API calls can be executed from javascript that is not running
/// on the same host as the API server.
///
/// # TODO
///
/// * Add the Access-Control-Allow-Credentials header -- it needs an actual domain for Origin in
///   order to work.
///
pub fn enable_cors<'mw>(_req: &mut Request<BDCSConfig>, mut res: Response<'mw, BDCSConfig>) -> MiddlewareResult<'mw, BDCSConfig> {
    // Set appropriate headers
    res.set(header::AccessControlAllowOrigin::Any);
    res.set(header::AccessControlAllowHeaders(vec![
        // Hyper uses the `unicase::Unicase` type to ensure comparisons are done
        // case-insensitively. Here, we use `into()` to convert to one from a `&str`
        // so that we don't have to import the type ourselves.
        "Origin".into(),
        "X-Requested-With".into(),
        "Content-Type".into(),
        "Accept".into(),
    ]));

    // Pass control to the next middleware
    res.next_middleware()
}
