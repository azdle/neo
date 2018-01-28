#![recursion_limit = "1024"]
#[macro_use]
extern crate error_chain;
extern crate neo;
extern crate reqwest;
extern crate clap;
extern crate pretty_env_logger;
#[macro_use] extern crate log;
extern crate config as config_lib;
extern crate app_dirs;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate rpassword;
extern crate rprompt;

use app_dirs::AppInfo;
use clap::{Arg, App, SubCommand};

// Note that this is different than the errors module in lib.rs
mod errors {
    error_chain!{
        links {
            Neo(::neo::errors::Error, ::neo::errors::ErrorKind);
        }

        foreign_links {
            Config(::config_lib::ConfigError);
            AppDirs(::app_dirs::AppDirsError);
        }
    }
}

use errors::*;

const APP_INFO: AppInfo = AppInfo{name: "neo", author: "azdle"};

fn main() {
    pretty_env_logger::init();

    trace!("main() [neo]");

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
    trace!("run()");
    let app_config = config::Config::build()?;
    let default_site = app_config.default_site;

    debug!("defualt site: {:?}", default_site);

    let matches = App::new(env!("CARGO_PKG_NAME"))
                          .version(env!("CARGO_PKG_VERSION"))
                          .author(env!("CARGO_PKG_AUTHORS"))
                          .about("CLI interface for managing Neocities websites (https://neocities.org)")
                          .arg(Arg::with_name("site")
                               .short("s")
                               .help("Set site name explicitly")
                               .required(false)
                               .takes_value(true))
                          .arg(Arg::with_name("user")
                               .short("u")
                               .help("Set a username different from site name")
                               .required(false)
                               .takes_value(true))
                          .arg(Arg::with_name("password")
                               .short("p")
                               .help("Provide password explicitly (will prompt if omitted)")
                               .required(false)
                               .takes_value(true))
                          .arg(Arg::with_name("verbose")
                               .short("v")
                               .multiple(true)
                               .help("Sets the level of verbosity (max 4)"))
                          .arg(Arg::with_name("no-interactive")
                               .short("n")
                               .help("Don't attempt to prompt for user or password, just fail"))
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

    // TODO: Set verbosity manually
    match matches.occurrences_of("verbose") {
        0 => warn!("Verbosity: WARN"),
        1 => info!("Verbosity: INFO"),
        2 => debug!("Verbosity: DEBUG"),
        3 => trace!("Verbosity: TRACE"),
        _ => println!("Don't be crazy"),
    }

    let no_interactive = matches.is_present("no-interactive");

    let site = if let Some(site) = matches.value_of("site") {
        site.to_owned()
    } else if let Some(site) = default_site {
        site
    } else if no_interactive {
        panic!("no site")
    } else {
        if let Ok(site) = rprompt::prompt_reply_stdout("site: ") {
            site
        } else {
            panic!("no site")
        }
    };
    debug!("site: {}", site);

    let auth = if let Some(password) = matches.value_of("password") {
        neo::site::Auth::Password(
            neo::site::Password{
                user: site.clone(),
                password: password.to_owned()
            }
        )
    } else if let Some(auth) = app_config.sites.get(&site) {
        match auth {
            &config::Auth::Password(ref password) => {
                neo::site::Auth::Password(
                    neo::site::Password{
                        user: site.clone(),
                        password: password.password.to_owned()
                    }
                )
            },
            &config::Auth::Key(ref key) => {
                neo::site::Auth::Key(
                    neo::site::Key { key: key.key.to_owned() }
                )
            },
        }
    } else if no_interactive {
        panic!("no password")
    } else {
        panic!("unimplemented");
        //debug!("will prompt for password");
        //if let Ok(password) = rpassword::prompt_password_stdout("password: *typing not shown*") {
        //    debug!("got {}", password);
        //    Ok(password)
        //} else {
        //    Err("needs password")
        //}
    };
    debug!("auth: {:?}", auth);

    let site = neo::Site::new(auth);

    match matches.subcommand() {
        ("info", _) => {
            let info = site.info()?;
            println!("{:?}", info);
        },
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

pub mod config {
    use std::collections::BTreeMap;
    #[derive(Deserialize, Debug)]
    #[serde(untagged)]
    pub enum Auth {
        Key(Key),
        Password(Password),
    }
    #[derive(Deserialize, Debug)]
    pub struct Key {
        pub key: String,
    }
    #[derive(Deserialize, Debug)]
    pub struct Password {
        pub password: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct Config {
        pub default_site: Option<String>,
        pub sites: BTreeMap<String, Auth>,
    }

    impl Config {
        pub fn build() -> Result<Self, ::config_lib::ConfigError> {
            use app_dirs::*;
            use std::path::PathBuf;

            trace!("Config::build()");

            let mut s = ::config_lib::Config::new();

            let global_config_path = {
                let mut path = app_root(AppDataType::UserConfig, &super::APP_INFO).unwrap();
                path.push("conf.toml");
                path
            };

            s.merge(::config_lib::File::from(global_config_path).required(false))?;

            let mut local_config_path = PathBuf::from(".").canonicalize().unwrap();
            local_config_path.push("Neo.toml"); // push initial filename
            while { // this is a "do {} while ()" loop.
                let config_path_attempt = PathBuf::from(&local_config_path).with_file_name("Neo.toml");
                trace!("Checking {}.", config_path_attempt.to_string_lossy());
                if config_path_attempt.exists() {
                    info!("Found config file at {}", config_path_attempt.to_string_lossy());
                    s.merge(::config_lib::File::with_name("Neo.toml").required(false))?;
                    false // break
                } else {
                    local_config_path.pop()
                }
            } {}

            // You can deserialize (and thus freeze) the entire configuration as
            match s.try_into() {
                Err(_) => Ok(Config { default_site: None, sites: BTreeMap::new() }),
                c => c
            }
        }
    }
}
