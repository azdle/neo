#![recursion_limit = "1024"]
#[macro_use]
extern crate error_chain;
extern crate neo;
extern crate reqwest;
extern crate clap;
extern crate pretty_env_logger;
#[macro_use] extern crate log;

use clap::{Arg, App, SubCommand};

// Note that this is different than the errors module in lib.rs
mod errors {
    error_chain!{
        links {
            Neo(::neo::errors::Error, ::neo::errors::ErrorKind);
        }
    }
}

use errors::*;

fn main() {
    pretty_env_logger::init();

    if let Err(ref e) = run() {
        use std::io::Write;
        let stderr = &mut ::std::io::stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "error: {}", e).expect(errmsg);

        for e in e.iter().skip(1) {
            writeln!(stderr, "caused by: {}", e).expect(errmsg);
        }

        if let Some(backtrace) = e.backtrace() {
            writeln!(stderr, "backtrace: {:?}", backtrace).expect(errmsg);
        }

        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
                          .version(env!("CARGO_PKG_VERSION"))
                          .author(env!("CARGO_PKG_AUTHORS"))
                          .about("CLI interface for managing Neocities websites (https://neocities.org)")
                          .arg(Arg::with_name("site")
                               .short("s")
                               .help("Set site name explicitly")
                               .required(true)
                               .takes_value(true))
                          .arg(Arg::with_name("verbose")
                               .short("v")
                               .multiple(true)
                               .help("Sets the level of verbosity (max 4)"))
                          .subcommand(SubCommand::with_name("info")
                                      .about("Fetch site info"))
                          .subcommand(SubCommand::with_name("list")
                                      .about("List site files"))
                          .subcommand(SubCommand::with_name("upload")
                                      .about("Upload file to site")
                                      .arg(Arg::with_name("PATH")
                                          .help("The local path of the file to upload")
                                          .required(true)
                                          .index(1)))
                          .subcommand(SubCommand::with_name("delete")
                                      .about("Delete file from site")
                                      .arg(Arg::with_name("FILE")
                                          .help("The remote file to delete")
                                          .required(true)
                                          .index(1)))
                          .get_matches();

    let site_name = matches.value_of("site").unwrap();
    debug!("site: {}", site_name);

    match matches.occurrences_of("verbose") {
        0 => warn!("Verbosity: WARN"),
        1 => info!("Verbosity: INFO"),
        2 => debug!("Verbosity: DEBUG"),
        3 => trace!("Verbosity: TRACE"),
        _ => println!("Don't be crazy"),
    }

    match matches.subcommand() {
        ("info", _) => {},
        ("list", _) => {},
        ("upload", Some(matches)) => {
            let path_str = matches.value_of("PATH").unwrap();
            info!("upload: {}", path_str);
        },
        ("delete", Some(matches)) => {
            let file_str = matches.value_of("FILE").unwrap();
            info!("delete: {}", file_str);
        },
        _ => { println!("{}", matches.usage()) },
    }

    Ok(())
}
