pub mod v0;


use hyper::header;
use nickel::{Request, Response, MiddlewareResult};

/// Enable CORS support
/// https://developer.mozilla.org/en-US/docs/Web/HTTP/Access_control_CORS
pub fn enable_cors<'mw>(_req: &mut Request, mut res: Response<'mw>) -> MiddlewareResult<'mw> {
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
