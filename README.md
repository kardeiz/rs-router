rs-router
=========

A [RegexSet](https://doc.rust-lang.org/regex/regex/struct.RegexSet.html) based router for use with stable Hyper (0.10.x).

The `rs` stands for `RegexSet`, not Rust&mdash;

Similar to and inspired by [reroute](https://github.com/gsquire/reroute), but potentially faster (no unnecessary string allocations, no hashmaps, and method-first-matching).

Provides light wrappers around `hyper::server::Request` and `hyper::server::Response` 

* to provide some convenience methods, like `req.captures()`, which provides the captures of the matching `Regex`, 
* and to enable nice request handlers with signature `fn(req: Request) -> Result<Response, Error>` where `Error` implements `Into<Response>` (which allows you to bail out of the handler early on errors).

See [/examples/simple.rs](/examples/simple.rs) for usage.

