extern crate hyper;
extern crate rs_router;
extern crate regex;
extern crate num_cpus;
#[macro_use]
extern crate lazy_static;

use std::io::Read;

use hyper::server::{Server, Request, Response, Handler};
use regex::Regex;
use rs_router::{Router, utils as rs_utils};

fn digit_handler(req: Request, res: Response, regex: &Regex) {
    let digits = regex.captures(rs_utils::extract_path(&req.uri))
        .and_then(|c| c.at(1) )
        .unwrap();
    if digits.len() > 5 {
        res.send(b"a big number!").unwrap();
    } else {
        res.send(b"not a big number").unwrap();
    }
}

fn body_handler(mut req: Request, res: Response, _: &Regex) {
    let mut body = String::new();
    let _ = req.read_to_string(&mut body);
    res.send(body.as_bytes()).unwrap();
}

fn not_found(req: Request, res: Response) {
    let uri = format!("{}", req.uri);
    let message = format!("why you calling {}?", uri);
    res.send(message.as_bytes()).unwrap();
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
        .add_get(r"/(\d+)", digit_handler)
        .add_post(r"/body", body_handler)
        .not_found(not_found)
        .finish()
        .unwrap();

    server.handle_threads(router, 8 * ::num_cpus::get()).unwrap();
}