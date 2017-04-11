// #![feature(question_mark)]

extern crate regex;
extern crate hyper;

#[macro_use]
extern crate quick_error;

#[macro_use]
extern crate log;

pub mod err {
    quick_error! {
        #[derive(Debug)]
        pub enum Error {
            Other(err: Box<::std::error::Error + Send + Sync>) {
                from(e: &'static str) -> (e.into())
                from(e: ::std::io::Error) -> (e.into())
                description(err.description())
                display("{}", err)
            }
            NotFoundNotSet {
                description("Did not set not_found handler")
                display("{}", "Did not set not_found handler")
            }
            Regex(err: ::regex::Error) {
                from()
                description("Regex error")
                display("{}", err)
                cause(err)
            }            
        }
    }
    pub type Result<T> = ::std::result::Result<T, Error>;
}

use regex::{Regex, RegexSet, Captures};

use hyper::server::{Handler, Response as HyperResponse, Request as HyperRequest};
use hyper::method::Method;
use hyper::uri::RequestUri;
use hyper::status::StatusCode;
use hyper::header::{self, Headers};
use hyper::net::Fresh;

use std::ops::{Deref, DerefMut};
use std::convert::From;
use std::io::Write;

pub struct RequestExtensions<'a> {
    path_delims: Option<(usize, Option<usize>)>,
    regex_match: Option<&'a Regex>
}


pub struct Request<'a, 'b: 'a, 'c> {
    inner: HyperRequest<'a, 'b>,
    extensions: RequestExtensions<'c>
}


impl<'a, 'b: 'a, 'c> Deref for Request<'a, 'b, 'c> {
    type Target = HyperRequest<'a, 'b>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, 'b: 'a, 'c> DerefMut for Request<'a, 'b, 'c> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<'a, 'b: 'a, 'c> From<HyperRequest<'a, 'b>> for Request<'a, 'b, 'c> {

    fn from(x: HyperRequest<'a, 'b>) -> Self {        
        let path_delims = match x.uri {            
            ::hyper::uri::RequestUri::AbsolutePath(ref s) => {                
                if let Some(pos) = s.find('?') {
                    Some((pos, Some(pos+1)))
                } else {
                    Some((s.len(), None))
                }
            },
            _ => None
        };

        let extensions = RequestExtensions {
            path_delims: path_delims,
            regex_match: None
        };

        Request {
            inner: x,
            extensions: extensions
        }
    }

}

impl<'a, 'b: 'a, 'c> Request<'a, 'b, 'c> {

    pub fn path(&self) -> &str {
        match self.inner.uri {
            RequestUri::AbsolutePath(ref s) => {
                let pos = 
                    self.extensions.path_delims.unwrap().0;
                &s[..pos]
            },
            RequestUri::AbsoluteUri(ref url) => url.path(),
            _ => panic!("Unexpected request URI")
        }
    }

    pub fn query(&self) -> Option<&str> {
        match self.inner.uri {
            RequestUri::AbsolutePath(ref s) => {
                self.extensions.path_delims.unwrap().1
                    .map(|pos| &s[pos..] )
            },
            RequestUri::AbsoluteUri(ref url) => url.query(),
            _ => panic!("Unexpected request URI")
        }
    }

    pub fn captures(&self) -> Option<Captures> {
        self.extensions.regex_match
            .and_then(|x| x.captures(self.path()))            
    }

}

pub trait WriteBody: Send {
    fn write_body(&mut self, res: &mut Write) -> ::std::io::Result<()>;
}

impl WriteBody for Vec<u8> {
    fn write_body(&mut self, res: &mut Write) -> ::std::io::Result<()> {
        self.as_slice().write_body(res)
    }
}

impl<'a> WriteBody for &'a [u8] {
    fn write_body(&mut self, res: &mut Write) -> ::std::io::Result<()> {
        res.write_all(self)
    }
}

impl WriteBody for String {
    fn write_body(&mut self, res: &mut Write) -> ::std::io::Result<()> {
        self.as_bytes().write_body(res)
    }
}

impl<'a> WriteBody for &'a str {
    fn write_body(&mut self, res: &mut Write) -> ::std::io::Result<()> {
        self.as_bytes().write_body(res)
    }
}

impl WriteBody for ::std::fs::File {
    fn write_body(&mut self, res: &mut Write) -> ::std::io::Result<()> {
        ::std::io::copy(self, res).map(|_| ())
    }
}

pub struct Response {
    pub status: Option<StatusCode>,
    pub headers: Headers,
    pub body: Option<Box<WriteBody>>
}

impl Response {

    pub fn new() -> Self {
        Response {
            status: None,
            headers: Headers::new(),
            body: None
        }
    }

    pub fn write_back(self, mut res: HyperResponse<Fresh>) {

        fn write_body(mut res: HyperResponse<Fresh>, body: Option<Box<WriteBody>>) 
            -> ::std::io::Result<()> {
            match body {
                Some(mut b) => {
                    let mut raw = res.start()?;
                    b.write_body(&mut raw)?;
                    Ok(raw.end()?)
                },
                None => {
                    res.headers_mut().set(header::ContentLength(0));
                    Ok(res.start()?.end()?)
                }
            }
        }

        *res.headers_mut() = self.headers;
        *res.status_mut() = self.status.unwrap_or(StatusCode::Ok);
        
        if let Err(e) = write_body(res, self.body) {
            error!("Error writing response: {}", e);
        }
    }

}

pub trait InnerHandler: Send + Sync {
    fn handle<'a, 'b, 'c>(&'a self, Request<'a, 'b, 'c>) -> Response;
}

impl<F, E> InnerHandler for F where E: Into<Response>, F: Fn(Request) -> Result<Response, E> + Sync + Send {
    fn handle<'a, 'b, 'c>(&'a self, req: Request<'a, 'b, 'c>) -> Response {
        self(req).unwrap_or_else(|e| e.into() )
    }
}

impl Router {
    pub fn build<'a>() -> RouterBuilder<'a> { RouterBuilder::default() }
}

unsafe impl Send for Router {}
unsafe impl Sync for Router {}

impl<'a> RouterBuilder<'a> {
    pub fn add_not_found<B>(mut self, handler: B) -> Self where B: InnerHandler + 'static {
        self.not_found = Some(Box::new(handler));
        self
    }
}

macro_rules! impls {
    ($([$prefix_regex_set:ident, 
        $prefix_regexes:ident, 
        $prefix_priorities:ident, 
        $prefix_handlers:ident, 
        $prefix_strs:ident, 
        $he:pat, 
        $add:ident,
        $add_priority:ident]),+) => {
        
        pub struct Router {
            $(
                $prefix_regex_set: Option<RegexSet>,
                $prefix_regexes: Option<Vec<Regex>>,
                $prefix_priorities: Option<Vec<usize>>,
                $prefix_handlers: Option<Vec<Box<InnerHandler>>>,
            )+
            not_found: Box<InnerHandler>
        }

        impl Handler for Router {
            fn handle<'a, 'k>(&'a self, req: HyperRequest<'a, 'k>, res: HyperResponse<'a>) {
                let mut req = Request::from(req);
                match req.method {
                    $(
                        $he => {
                            if let Some(i) = self.$prefix_regex_set
                                .iter()
                                .flat_map(|s| s.matches(req.path()) )
                                .min_by(|x, y| 
                                    (&self.$prefix_priorities.as_ref().unwrap()[*x])
                                        .cmp(&self.$prefix_priorities.as_ref().unwrap()[*y])
                                ) {
                                let handler = 
                                    &self.$prefix_handlers.as_ref().unwrap()[i];
                                let regex = 
                                    &self.$prefix_regexes.as_ref().unwrap()[i];
                                req.extensions.regex_match = Some(regex);
                                return handler.handle(req).write_back(res);
                            }                                
                        },
                    )+
                    _ => { }
                }
                self.not_found.handle(req).write_back(res);
            }
        }

        #[derive(Default)]
        pub struct RouterBuilder<'a> {
            $(
                $prefix_strs: Option<Vec<&'a str>>,
                $prefix_priorities: Option<Vec<usize>>,
                $prefix_handlers: Option<Vec<Box<InnerHandler>>>,
            )+
            not_found: Option<Box<InnerHandler>>
        }

        impl<'a> RouterBuilder<'a> {
            $(
                pub fn $add<B>(mut self, re: &'a str, handler: B) -> Self
                    where B: InnerHandler + 'static {
                    self.$prefix_strs = self.$prefix_strs.or(Some(Vec::new()));
                    self.$prefix_handlers = self.$prefix_handlers.or(Some(Vec::new()));
                    self.$prefix_priorities = self.$prefix_priorities.or(Some(Vec::new()));
                    
                    self.$prefix_strs.as_mut().unwrap().push(re);
                    self.$prefix_handlers.as_mut().unwrap().push(Box::new(handler));
                    self.$prefix_priorities.as_mut().unwrap().push(0);
                    self
                }
            )+

            $(
                pub fn $add_priority<B>(mut self, re: &'a str, priority: usize, handler: B) -> Self
                    where B: InnerHandler + 'static {
                    self.$prefix_strs = self.$prefix_strs.or(Some(Vec::new()));
                    self.$prefix_handlers = self.$prefix_handlers.or(Some(Vec::new()));
                    self.$prefix_priorities = self.$prefix_priorities.or(Some(Vec::new()));
                    
                    self.$prefix_strs.as_mut().unwrap().push(re);
                    self.$prefix_handlers.as_mut().unwrap().push(Box::new(handler));
                    self.$prefix_priorities.as_mut().unwrap().push(priority);
                    self
                }
            )+

            pub fn finish(self) -> ::err::Result<Router> {                
                $(
                    let mut $prefix_regex_set = None;
                    let mut $prefix_regexes = None;

                    if let Some(ss) = self.$prefix_strs {
                        $prefix_regex_set = Some(RegexSet::new(ss.iter())?);
                        
                        $prefix_regexes = {
                            let mut out = Vec::new();
                            for s in &ss {
                                out.push(Regex::new(s)?);
                            }
                            Some(out)
                        };                            
                    }
                )+

                let out = Router {
                    $(
                        $prefix_regex_set: $prefix_regex_set,
                        $prefix_regexes: $prefix_regexes,
                        $prefix_handlers: self.$prefix_handlers,
                        $prefix_priorities: self.$prefix_priorities,
                    )+
                    not_found: self.not_found.ok_or(err::Error::NotFoundNotSet)?
                };
                Ok(out)
            }
        }
    }
}

impls!{
    [get_regex_set, get_regexes, get_priorities, get_handlers, get_strs, Method::Get, add_get, add_get_with_priority],
    [post_regex_set, post_regexes, post_priorities, post_handlers, post_strs, Method::Post, add_post, add_post_with_priority],
    [put_regex_set, put_regexes, put_priorities, put_handlers, put_strs, Method::Put, add_put, add_put_with_priority],
    [patch_regex_set, patch_regexes, patch_priorities, patch_handlers, patch_strs, Method::Patch, add_patch, add_patch_with_priority],
    [delete_regex_set, delete_regexes, delete_priorities, delete_handlers, delete_strs, Method::Delete, add_delete, add_delete_with_priority],
    [head_regex_set, head_regexes, head_priorities, head_handlers, head_strs, Method::Head, add_head, add_head_with_priority]
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
