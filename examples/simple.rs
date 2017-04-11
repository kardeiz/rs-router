extern crate hyper;
extern crate rs_router;
extern crate regex;
extern crate num_cpus;
#[macro_use]
extern crate lazy_static;

use std::io::Read;

use hyper::server::{Server};
use rs_router::{Router, Request, Response};

pub enum Error {
    Warning(&'static str)
}

impl From<&'static str> for Error {
    fn from(t: &'static str) -> Error {
        Error::Warning(t)
    }
}

impl From<Error> for Response {
    fn from(t: Error) -> Response {
        match t {
            Error::Warning(msg) => {
                let mut out = Response::new();
                out.status = Some(::hyper::status::StatusCode::BadRequest);
                out.body = Some(Box::new(msg));
                out
            }
        }        
    }
}

fn maybe_digits_handler(req: Request) -> Result<Response, Error> {
    let digits: u64 = req.captures()
        .and_then(|c| c.get(1) )
        .map(|x| x.as_str() )
        .unwrap()
        .parse()
        .map_err(|_| "Could not parse as u64")?;

    let mut out = Response::new();
    let msg = format!("Requested digits: {}", digits);
    out.body = Some(Box::new(msg));
    Ok(out)
}

fn digit_handler(req: Request) -> Result<Response, Error> {
    let digits = req.captures()
        .and_then(|c| c.get(1) )
        .map(|x| x.as_str() )
        .unwrap();
    let mut out = Response::new();
    let msg = format!("Requested digits: {}", digits);
    out.body = Some(Box::new(msg));
    Ok(out)
}

fn body_handler(mut req: Request) -> Result<Response, Error>{
    let mut out = Response::new();
    let mut msg = String::new();
    let _ = req.read_to_string(&mut msg);
    out.body = Some(Box::new(msg));
    Ok(out)    
}

fn index(_: Request) -> Result<Response, Error> {
    let mut out = Response::new();
    out.body = Some(Box::new("Requested /"));
    Ok(out)
}

fn not_found(req: Request) -> Result<Response, Error> {
    let mut out = Response::new();
    out.status = Some(::hyper::status::StatusCode::NotFound);
    let msg = format!("Requested path: {}, but no handler found", req.path());
    out.body = Some(Box::new(msg));
    Ok(out)
}

fn main() {

    let server = {        
        use std::time::Duration;

        let host = ::std::env::var("WEB_HOST")
            .unwrap_or("0.0.0.0".into());
        let port = ::std::env::var("WEB_PORT")
            .ok()
            .as_ref()
            .and_then(|x| x.parse().ok() )
            .unwrap_or(3000u16);

        let mut server = Server::http((&host as &str, port)).unwrap();
        server.keep_alive(Some(Duration::from_secs(5)));
        server.set_read_timeout(Some(Duration::from_secs(30)));
        server.set_write_timeout(Some(Duration::from_secs(1)));
        server
    };

    let router = Router::build()
        .add_get(r"\A/\z", index)
        .add_get(r"\A/(\d+)\z", digit_handler)
        .add_get(r"\A/maybe_digits/(.+)\z", maybe_digits_handler)
        .add_post(r"\A/body\z", body_handler)
        .add_not_found(not_found)
        .finish()
        .unwrap();

    server.handle_threads(router, 8 * ::num_cpus::get()).unwrap();
}