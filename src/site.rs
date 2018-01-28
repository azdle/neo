use reqwest;
use std::path::PathBuf;

use errors::*;

const USER_AGENT: &'static str = concat!("neo/",  env!("CARGO_PKG_VERSION"));

#[derive(Serialize, Deserialize, Debug)]
enum ApiResult {
    Info(InfoResult),
    List(ListResult),
    Error(ErrorResult),
}

#[derive(Serialize, Deserialize, Debug)]
struct InfoResult {
    result: String,
    info: Info,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorResult {
    pub result: String,
    pub error_type: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Info {
    pub sitename: String,
    pub views: i64,
    pub hits: i64,
    pub created_at: String,
    pub last_updated: Option<String>,
    pub domain: Option<String>,
    pub tags: Vec<String>,
    pub latest_ipfs_hash: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ListResult {
    result: String,
    files: Vec<File>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct File {
    pub path: String,
    pub is_directory: bool,
    pub size: i64,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Auth {
    Password(Password),
    Key(Key),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Key {
    pub key: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Password {
    pub user: String,
    pub password: String,
}

#[derive(Debug)]
pub struct Site {
    auth: Auth,
    client: reqwest::Client,
}

impl Site {
    pub fn new(auth: Auth) -> Site {
        trace!("Site::new()");
        let client = reqwest::Client::new();

        Site { auth, client }
    }

    pub fn with_key(key: String) -> Site {
        trace!("Site::with_key()");
        let client = reqwest::Client::new();

        Site {
            auth: Auth::Key(Key {
                key,
            }),
            client,
        }
    }

    pub fn with_password(user: String, password: String) -> Site {
        trace!("Site::with_password()");
        let client = reqwest::Client::new();

        Site {
            auth: Auth::Password(Password {
                user,
                password,
            }),
            client,
        }
    }

    pub fn info(&self) -> Result<Info> {
        trace!("Site::info()");
        use reqwest::header::{Authorization, Basic, Bearer, UserAgent};

        let mut request = self.client.get("https://neocities.org/api/info");

        request.header(UserAgent::new(USER_AGENT));

        match self.auth {
            Auth::Key(ref key) => {
                debug!("auth with bearer token");
                request.header( Authorization( Bearer { token: key.key.clone() }))
            },
            Auth::Password(ref password) => {
                debug!("auth with password");
                request.header( Authorization( Basic {
                    username: password.user.clone(),
                    password: Some(password.password.clone()),
                 }))
            },
        };

        debug!("request: {:?}", request);

        let mut response = request.send()
            .expect("Failed to send request");

        debug!("response: {:?}", response);


        if response.status().is_success() {
            let r: InfoResult = response.json().unwrap();
            Ok(r.info)
        } else {
            let r: ::std::result::Result<ErrorResult, ::reqwest::Error> = response.json();
            match r {
                Ok(r) => Err(ErrorKind::ServerError(r).into()),
                _ => Err(ErrorKind::UnexpectedResponse(response).into()),
            }
        }
    }

    pub fn list(&self) -> Result<Vec<File>> {
        trace!("Site::list()");
        use reqwest::header::{Authorization, Basic, Bearer, UserAgent};

        let mut request = self.client.get("https://neocities.org/api/list");

        request.header(UserAgent::new(USER_AGENT));

        match self.auth {
            Auth::Key(ref key) => {
                debug!("auth with bearer token");
                request.header( Authorization( Bearer { token: key.key.clone() }))
            },
            Auth::Password(ref password) => {
                debug!("auth with password");
                request.header( Authorization( Basic {
                    username: password.user.clone(),
                    password: Some(password.password.clone()),
                 }))
            },
        };

        debug!("request: {:?}", request);

        let mut response = request.send()
            .expect("Failed to send request");

        debug!("response: {:?}", response);

        if response.status().is_success() {
            let r: ListResult = response.json().unwrap();
            Ok(r.files)
        } else {
            let r: ::std::result::Result<ErrorResult, ::reqwest::Error> = response.json();
            match r {
                Ok(r) => Err(ErrorKind::ServerError(r).into()),
                _ => Err(ErrorKind::UnexpectedResponse(response).into()),
            }
        }
    }

    pub fn upload(&self, path: String, file: PathBuf) -> Result<()> {
        trace!("Site::upload()");
        use reqwest::header::{Authorization, Basic, Bearer, UserAgent};

        let form = reqwest::multipart::Form::new()
            .file(path, file).unwrap();

        let url = "https://neocities.org/api/upload";

        let mut request = self.client.post(url);

        request.header(UserAgent::new(USER_AGENT));

        match self.auth {
            Auth::Key(ref key) => {
                debug!("auth with bearer token");
                request.header( Authorization( Bearer { token: key.key.clone() }))
            },
            Auth::Password(ref password) => {
                debug!("auth with password");
                request.header( Authorization( Basic {
                    username: password.user.clone(),
                    password: Some(password.password.clone()),
                 }))
            },
        };

        request.multipart(form);

        debug!("request: {:?}", request);

        let mut response = request.send()
            .expect("Failed to send request");

        debug!("response: {:?}", response);

        debug!("response: {:?}", response);

        if response.status().is_success() {
            Ok(())
        } else {
            let r: ::std::result::Result<ErrorResult, ::reqwest::Error> = response.json();
            match r {
                Ok(r) => Err(ErrorKind::ServerError(r).into()),
                _ => Err(ErrorKind::UnexpectedResponse(response).into()),
            }
        }
    }

    pub fn delete(&self, files: Vec<String>) -> Result<()> {
        trace!("Site::delete()");
        use reqwest::header::{Authorization, Basic, Bearer, UserAgent};

        let mut query = String::new();

        for file in files {
            query.push_str("filenames[]=");
            query.push_str(&file);
            query.push('&');
        }

        let url = format!("https://neocities.org/api/delete?{}", query);

        let mut request = self.client.post(&url);

        request.header(UserAgent::new(USER_AGENT));

        match self.auth {
            Auth::Key(ref key) => {
                debug!("auth with bearer token");
                request.header( Authorization( Bearer { token: key.key.clone() }))
            },
            Auth::Password(ref password) => {
                debug!("auth with password");
                request.header( Authorization( Basic {
                    username: password.user.clone(),
                    password: Some(password.password.clone()),
                 }))
            },
        };

        debug!("request: {:?}", request);

        let mut response = request.send()
            .expect("Failed to send request");

        debug!("response: {:?}", response);

        if response.status().is_success() {
            Ok(())
        } else {
            let r: ::std::result::Result<ErrorResult, ::reqwest::Error> = response.json();
            match r {
                Ok(r) => Err(ErrorKind::ServerError(r).into()),
                _ => Err(ErrorKind::UnexpectedResponse(response).into()),
            }
        }
    }
}
