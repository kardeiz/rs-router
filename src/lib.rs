// #![feature(question_mark)]

extern crate regex;
extern crate hyper;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate quick_error;

pub mod err {
    quick_error! {
        #[derive(Debug)]
        pub enum Error {
            Other(err: Box<::std::error::Error + Send + Sync>) {
                from(e: &'static str) -> (e.into())
                description(err.description())
                display("{}", err)
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

use hyper::server::{Handler, Response, Request as HyperRequest};
use hyper::method::Method;
use hyper::uri::RequestUri;


use std::ops::{Deref, DerefMut};
use std::convert::From;
use std::cell::Cell;


pub struct Request<'a, 'b: 'a, 'c> {
    inner: HyperRequest<'a, 'b>,
    delims: Cell<Option<(usize, Option<usize>)>>,
    regex_match: Option<&'c Regex>
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
        Request {
            inner: x,
            delims: Cell::new(None),
            regex_match: None
        }
    }
}

impl<'a, 'b: 'a, 'c> Request<'a, 'b, 'c> {

    fn with_regex(mut self, regex: &'c Regex) -> Self {
        self.regex_match = Some(regex);
        self
    }

    pub fn path(&self) -> &str {
        match self.inner.uri {
            RequestUri::AbsolutePath(ref s) => {
                if let Some((pos, _)) = self.delims.get() {
                    &s[..pos]
                } else {
                    let delims = s.find('?')
                        .map(|pos| (pos, Some(pos + 1)))
                        .unwrap_or((s.len(), None));
                    self.delims.set(Some(delims));
                    &s[..delims.0]
                }
            },
            RequestUri::AbsoluteUri(ref url) => url.path(),
            _ => panic!("Unexpected request URI")
        }
    }

    pub fn query(&self) -> Option<&str> {
        match self.inner.uri {
            RequestUri::AbsolutePath(ref s) => {
                if let Some((_, opt_pos)) = self.delims.get() {
                    opt_pos.map(|pos| &s[pos..] )
                } else {
                    let delims = s.find('?')
                        .map(|pos| (pos, Some(pos + 1)))
                        .unwrap_or((s.len(), None));
                    self.delims.set(Some(delims));
                    delims.1.map (|pos| &s[pos..] )
                }
            },
            RequestUri::AbsoluteUri(ref url) => url.query(),
            _ => panic!("Unexpected request URI")
        }
    }

    pub fn captures(&self) -> Option<Captures> {
        self.regex_match
            .and_then(|x| x.captures(self.path()))
            
    }

}

pub trait InnerHandler: Send + Sync {
    fn handle<'a, 'b, 'c>(&'a self, Request<'a, 'b, 'c>, Response<'a>);

}

impl<F> InnerHandler for F where F: Fn(Request, Response) + Sync + Send {
    fn handle<'a, 'b, 'c>(&'a self, 
        req: Request<'a, 'b, 'c>, 
        res: Response<'a>) {
        self(req, res)
    }
}

impl Router {
     pub fn build<'a>() -> RouterBuilder<'a> { RouterBuilder::default() }
}

impl<'a> RouterBuilder<'a> {

    pub fn not_found<H: InnerHandler + 'static>(mut self, handler: H) -> Self {
        self.not_found = Some(Box::new(handler));
        self
    }

}

macro_rules! impls {
    ($([$prefix_regex_set:ident, 
        $prefix_regexes:ident, 
        $prefix_handlers:ident, 
        $prefix_strs:ident, 
        $he:pat, 
        $add:ident]),+) => {
        
        pub struct Router {
            $(
                $prefix_regex_set: Option<RegexSet>,
                $prefix_regexes: Option<Vec<Regex>>,
                $prefix_handlers: Option<Vec<Box<InnerHandler>>>,
            )+
            not_found: Box<InnerHandler>
        }

        unsafe impl Send for Router {}
        unsafe impl Sync for Router {}

        impl Handler for Router {
            fn handle<'a, 'k>(&'a self, req: HyperRequest<'a, 'k>, res: Response<'a>) {
                let req = Request::from(req);
                match req.method {
                    $(
                        $he => {
                            if let Some(i) = self.$prefix_regex_set
                                .iter()
                                .flat_map(|s| s.matches(req.path()) )
                                .next() {
                                let handler = 
                                    &self.$prefix_handlers.as_ref().unwrap()[i];
                                let regex = 
                                    &self.$prefix_regexes.as_ref().unwrap()[i];
                                let req = req.with_regex(regex);
                                handler.handle(req, res);
                                return;
                            }                                
                        },
                    )+
                    _ => { }
                }
                self.not_found.handle(req, res)
            }
        }

        #[derive(Default)]
        pub struct RouterBuilder<'a> {
            $(
                $prefix_strs: Option<Vec<&'a str>>,
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
                    self.$prefix_strs.as_mut().unwrap().push(re);
                    self.$prefix_handlers.as_mut().unwrap().push(Box::new(handler));
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
                    )+
                    not_found: self.not_found.ok_or("Must include not found")?
                };
                Ok(out)
            }
        }
    }
}

impls!{
    [get_regex_set, get_regexes, get_handlers, get_strs, Method::Get, add_get],
    [post_regex_set, post_regexes, post_handlers, post_strs, Method::Post, add_post],
    [put_regex_set, put_regexes, put_handlers, put_strs, Method::Put, add_put],
    [patch_regex_set, patch_regexes, patch_handlers, patch_strs, Method::Patch, add_patch],
    [delete_regex_set, delete_regexes, delete_handlers, delete_strs, Method::Delete, add_delete],
    [head_regex_set, head_regexes, head_handlers, head_strs, Method::Head, add_head]
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
