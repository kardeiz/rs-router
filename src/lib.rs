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

use regex::RegexSet;

use hyper::server::{Request, Response};
use hyper::method::Method;
use hyper::uri::RequestUri;

macro_rules! impls {
    ($([$prefix_re_set:ident, $prefix_handlers:ident, $prefix_strs:ident, $he:pat, $add:ident]),+) => {
        #[derive(Default)]
        pub struct Router {
            $(
                $prefix_re_set: Option<RegexSet>,
                $prefix_handlers: Option<Vec<Box<Fn(Request, Response)>>>,
            )+            
        }

        unsafe impl Send for Router {}
        unsafe impl Sync for Router {}

        impl Router {

            pub fn find_handler(&self, req: &Request) -> Option<&Box<Fn(Request, Response)>> {
                match req.method {
                    $(
                        $he => {
                            self.$prefix_re_set
                                .as_ref()
                                .iter()
                                .flat_map(|s| s.matches(Self::extract_path(&req.uri)))
                                .map(|i| &self.$prefix_handlers.as_ref().unwrap()[i] )
                                .next()
                        },
                    )+
                    _ => { None }
                }
            }

            pub fn extract_path(uri: &RequestUri) -> &str {
                match *uri {
                    RequestUri::AbsolutePath(ref s) => {                
                        s.find('?').map(|pos| &s[..pos] ).unwrap_or(&s[..])
                    },
                    RequestUri::AbsoluteUri(ref url) => url.path(),
                    _ => panic!("Unexpected request URI")
                }
            }

            pub fn build<'a>() -> RouterBuilder<'a> { RouterBuilder::default() }

        }

        #[derive(Default)]
        pub struct RouterBuilder<'a> {
            $(
                $prefix_strs: Option<Vec<&'a str>>,
                $prefix_handlers: Option<Vec<Box<Fn(Request, Response)>>>,
            )+
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
                    let mut $prefix_re_set = None;
                    if let Some(ss) = self.$prefix_strs {
                        $prefix_re_set = Some(RegexSet::new(ss.iter())?);
                    }
                )+

                let out = Router {
                    $(
                        $prefix_re_set: $prefix_re_set,
                        $prefix_handlers: self.$prefix_handlers,
                    )+
                };
                Ok(out)
            }
        }
    }
}

impls!{
    [get_re_set, get_handlers, get_strs, Method::Get, add_get],
    [post_re_set, post_handlers, post_strs, Method::Post, add_post],
    [put_re_set, put_handlers, put_strs, Method::Put, add_put],
    [patch_re_set, patch_handlers, patch_strs, Method::Patch, add_patch],
    [delete_re_set, delete_handlers, delete_strs, Method::Delete, add_delete],
    [head_re_set, head_handlers, head_strs, Method::Head, add_head],
    [options_re_set, options_handlers, options_strs, Method::Options, add_options]
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
