rs-router
=========

A [RegexSet](https://doc.rust-lang.org/regex/regex/struct.RegexSet.html) based router for use with stable Hyper (0.9.x).

Like [reroute](https://github.com/gsquire/reroute), but smaller and probably faster (no unnecessary string allocations, no hashmaps, and method-first-matching).

See [/examples/simple.rs](/examples/simple.rs) for usage.