#![recursion_limit = "1024"]
#[macro_use]
extern crate error_chain;

extern crate reqwest;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod site;
pub use site::Site;

pub mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain!{}
}

