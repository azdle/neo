#![recursion_limit = "1024"]
#[macro_use]
extern crate error_chain;

extern crate reqwest;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[macro_use]
extern crate log;

pub mod site;
pub use site::Site;

pub mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain!{
        errors {
            UnexpectedResponse(r: ::reqwest::Error) {
                description("unexpected network response")
                display("unexpected response: '{}'", r)
            }

            ServerError(r: ::site::ErrorResult) {
                description("error status received from server")
                display("[{}] {}", r.error_type, r.message)
            }
        }
    }
}

