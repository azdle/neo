#![recursion_limit = "1024"]
#[macro_use]
extern crate error_chain;
extern crate clap;
extern crate neo;
extern crate pretty_env_logger;
extern crate reqwest;
#[macro_use]
extern crate log;
extern crate app_dirs;
extern crate config as config_lib;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate rpassword;
extern crate rprompt;

use std::path::{Path, PathBuf};

use app_dirs::AppInfo;
use clap::{App, Arg, SubCommand};

// Note that this is different than the errors module in lib.rs
mod errors {
    error_chain! {
        links {
            Neo(::neo::errors::Error, ::neo::errors::ErrorKind);
        }

        foreign_links {
            Io(::std::io::Error);
            Config(::config_lib::ConfigError);
            AppDirs(::app_dirs::AppDirsError);
            StripPrefix(::std::path::StripPrefixError);
        }
    }
}

use errors::*;

const APP_INFO: AppInfo = AppInfo {
    name: "neo",
    author: "azdle",
};

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
        .arg(
            Arg::with_name("site")
                .short("s")
                .help("Set site name explicitly")
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("user")
                .short("u")
                .help("Set a username different from site name")
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("password")
                .short("p")
                .help("Provide password explicitly (will prompt if omitted)")
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity (max 4)"),
        )
        .arg(
            Arg::with_name("no-interactive")
                .short("n")
                .help("Don't attempt to prompt for user or password, just fail"),
        )
        .subcommand(SubCommand::with_name("info").about("Fetch site info"))
        .subcommand(
            SubCommand::with_name("list")
                .about("List site files")
                .alias("ls"),
        )
        .subcommand(
            SubCommand::with_name("upload")
                .about("Upload file to site")
                .arg(
                    Arg::with_name("FILE")
                        .help("The local file to upload")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("PATH")
                        .help("The remote path where the file is to be placed")
                        .required(false)
                        .index(2),
                ),
        )
        .subcommand(
            SubCommand::with_name("delete")
                .about("Delete file from site")
                .alias("rm")
                .arg(
                    Arg::with_name("PATH")
                        .help("The path of the remote file to delete")
                        .required(true)
                        .index(1),
                ),
        )
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
        neo::site::Auth::Password(neo::site::Password {
            user: site.clone(),
            password: password.to_owned(),
        })
    } else if let Some(auth) = app_config.sites.get(&site) {
        match auth {
            &config::Auth::Password(ref password) => {
                neo::site::Auth::Password(neo::site::Password {
                    user: site.clone(),
                    password: password.password.to_owned(),
                })
            }
            &config::Auth::Key(ref key) => neo::site::Auth::Key(neo::site::Key {
                key: key.key.to_owned(),
            }),
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
        }
        ("list", _) => {
            let list = site.list()?;
            println!("{:?}", list);
        }
        ("upload", Some(matches)) => {
            let root_path = { app_config.site_root.clone() };

            let file_str = matches
                .value_of("FILE")
                .expect("the required paramter FILE was somehow none");
            let path_str = match matches.value_of("PATH").map(|s| s.to_owned()) {
                Some(s) => s,
                None => match root_path {
                    Some(root_path) => {
                        let rel_path = to_root_relative_path(root_path.as_str(), file_str)?;
                        rel_path
                            .to_str()
                            .map(|s| s.to_owned())
                            .ok_or("invalid filename")?
                    }
                    None => file_str.to_owned(),
                },
            };

            debug!("upload: {} to {}", file_str, path_str);
            site.upload(path_str, file_str.into())?;
        }
        ("delete", Some(matches)) => {
            let root_path = { app_config.site_root.clone() };

            let path_str = matches
                .value_of("PATH")
                .expect("the required paramter PATH was somehow none");

            info!("delete: {}", path_str);

            let final_path = {
                if path_str.starts_with(":") {
                    // explicit path
                    path_str.get(1..).unwrap_or("").to_owned()
                } else {
                    match root_path {
                        Some(root_path) => {
                            let rel_path = to_root_relative_path(root_path.as_str(), path_str)?;
                            rel_path
                                .to_str()
                                .map(|s| s.to_owned())
                                .ok_or("invalid filename")?
                        }
                        None => path_str.to_owned(),
                    }
                }
            };

            info!("delete: {}", final_path);
            site.delete(vec![final_path])?;
        }
        _ => {
            println!("{}", matches.usage())
        }
    }

    Ok(())
}

fn to_root_relative_path<P: AsRef<Path>>(root_path: P, file_path: P) -> Result<PathBuf> {
    let root_path = root_path.as_ref().canonicalize()?;
    let file_path = file_path.as_ref().canonicalize()?;

    debug!("root: {}", root_path.to_string_lossy());
    debug!("file: {}", file_path.to_string_lossy());

    let rel = file_path
        .strip_prefix(&root_path)
        .map(|p| p.into())
        .map_err(|e| e.into());
    debug!("relative path {:?}", rel);
    rel
}

pub mod config {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

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
        // TODO: Why can't this be PathBuf?
        pub site_root: Option<String>,
        pub default_site: Option<String>,
        pub sites: BTreeMap<String, Auth>,
    }

    impl Config {
        pub fn build() -> Result<Self, ::config_lib::ConfigError> {
            use app_dirs::*;

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
            while {
                // this is a "do {} while ()" loop.
                let config_path_attempt =
                    PathBuf::from(&local_config_path).with_file_name("Neo.toml");
                trace!("Checking {}.", config_path_attempt.to_string_lossy());
                if config_path_attempt.exists() {
                    info!(
                        "Found config file at {}",
                        config_path_attempt.to_string_lossy()
                    );
                    s.merge(::config_lib::File::from(config_path_attempt).required(false))?;
                    local_config_path.pop();
                    s.set(
                        "site_root",
                        Some(local_config_path.to_string_lossy().into_owned()),
                    )?;
                    false // break
                } else {
                    local_config_path.pop()
                }
            } {}

            match s.try_into() {
                Err(_) => Ok(Config {
                    site_root: None,
                    default_site: None,
                    sites: BTreeMap::new(),
                }),
                c => c,
            }
        }
    }
}
