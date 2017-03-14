extern crate hyper;
extern crate rs_router;
extern crate regex;
extern crate num_cpus;
#[macro_use]
extern crate lazy_static;

use std::io::Read;

use hyper::server::{Server};
use rs_router::{Router, Request, Response};

// fn digit_handler(req: Request, res: Response) {
//     let digits = req.captures()
//         .and_then(|c| c.at(1) )
//         .unwrap();
//     if digits.len() > 5 {
//         res.send(b"a big number!").unwrap();
//     } else {
//         res.send(b"not a big number").unwrap();
//     }
// }

// fn body_handler(mut req: Request, res: Response) {
//     let mut body = String::new();
//     let _ = req.read_to_string(&mut body);
//     res.send(body.as_bytes()).unwrap();
// }

lazy_static! {
    static ref FOO_BAR: String = String::from("YES! FOO!");
}

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
                out.body = Some(Box::new(msg.as_bytes() as &[u8]));
                out
            }
        }        
    }

}


fn index(req: Request) -> Result<Response, Error> {
    if req.path() == "/foo" {
        let mut out = Response::new();
        out.status = Some(::hyper::status::StatusCode::Ok);
        out.body = Some(Box::new("FOO BAR".to_string()));
        Ok(out)
    } else {
        Err("Bad query".into())
    }
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
        .add_get(r"\A/.*\z", index)
        // .add_get(r"\A/(\d+)\z", digit_handler)
        // .add_post(r"\A/body\z", body_handler)
        .finish()
        .unwrap();

    server.handle_threads(router, 8 * ::num_cpus::get()).unwrap();
}