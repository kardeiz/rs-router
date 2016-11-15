#![feature(question_mark)]

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

use regex::{Regex, RegexSet};

use hyper::server::{Handler, Request, Response};
use hyper::method::Method;

pub struct UriParts<'a> {
    pub path: &'a str,
    pub query: Option<&'a str>
}

impl<'a> UriParts<'a> {
    pub fn new(path: &'a str, query: Option<&'a str>) -> Self {
        UriParts { path: path, query: query }
    }
}

pub struct Route<'a> {
    uri_parts: UriParts<'a>,
    regex: &'a Regex
}


pub mod utils {

    use super::UriParts;
    use hyper::uri::RequestUri;

    pub fn extract_parts(uri: &RequestUri) -> UriParts {
        match *uri {            
            ::hyper::uri::RequestUri::AbsolutePath(ref s) => {                
                if let Some(pos) = s.find('?') {
                    UriParts::new(&s[..pos], Some(&s[pos+1..]))
                } else {
                    UriParts::new(&s[..], None)
                }
            },
            ::hyper::uri::RequestUri::AbsoluteUri(ref url) => {
                UriParts::new(url.path(), url.query())
            },
            _ => panic!("Unexpected request URI")
        }
    }
}

impl Router {
     pub fn build<'a>() -> RouterBuilder<'a> { RouterBuilder::default() }
}

impl<'a> RouterBuilder<'a> {

    pub fn not_found<H: Handler + 'static>(mut self, handler: H) -> Self {
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
                $prefix_handlers: Option<Vec<Box<Fn(Request, Response)>>>,
            )+
            not_found: Box<Handler>
        }

        unsafe impl Send for Router {}
        unsafe impl Sync for Router {}

        impl Handler for Router {
            fn handle(&self, req: Request, res: Response) {
                let uri_parts = utils::extract_parts(&req.uri);
                match req.method {
                    $(
                        $he => {
                            if let Some(i) = self.$prefix_regex_set
                                .iter()
                                .flat_map(|s| s.matches(uri_parts.path) )
                                .next() {
                                let handler = 
                                    &self.$prefix_handlers.as_ref().unwrap()[i];
                                let regex = 
                                    &self.$prefix_regexes.as_ref().unwrap()[i];
                                let route = Route {
                                    uri_parts: uri_parts,
                                    regex: regex
                                };
                                // handler.handle(req, res, route);
                            }                                
                        },
                    )+
                    _ => { }
                }
                Handler::handle(*self.not_found, req, res)
            }
        }

        // impl Router {

        //     // pub fn find_handler(&self, req: &Request) -> Option<&Box<Fn(Request, Response)>> {
        //     //     match req.method {
        //     //         $(
        //     //             $he => {
        //     //                 self.$prefix_re_set
        //     //                     .as_ref()
        //     //                     .iter()
        //     //                     .flat_map(|s| s.matches(Self::extract_path(&req.uri)))
        //     //                     .map(|i| &self.$prefix_handlers.as_ref().unwrap()[i] )
        //     //                     .next()
        //     //             },
        //     //         )+
        //     //         _ => { None }
        //     //     }
        //     // }

        // }

        #[derive(Default)]
        pub struct RouterBuilder<'a> {
            $(
                $prefix_strs: Option<Vec<&'a str>>,
                $prefix_handlers: Option<Vec<Box<Fn(Request, Response)>>>,
            )+
            not_found: Option<Box<Handler>>
        }

        impl<'a> RouterBuilder<'a> {
            $(
                pub fn $add<B>(mut self, re: &'a str, handler: B) -> Self
                    where B: Fn(Request, Response) + Send + Sync + 'static {
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
    [head_regex_set, head_regexes, head_handlers, head_strs, Method::Head, add_head],
    [options_regex_set, options_regexes, options_handlers, options_strs, Method::Options, add_options]
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
