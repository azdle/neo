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

#[derive(Debug)]
pub struct Site {
    username: String,
    password: String,
    site: Option<String>,
    client: reqwest::Client,
}

impl Site {
    pub fn new(username: String, password: String, site: Option<String>) -> Site {
        trace!("Site::new()");
        let client = reqwest::Client::new();

        Site {
            username: username,
            password: password,
            site: site,
            client: client,
        }
    }

    pub fn info(&self) -> Result<Info> {
        trace!("Site::info()");
        use reqwest::header::{Authorization, Basic, UserAgent};

        let credentials = Basic {
            username: self.username.clone(),
            password: Some(self.password.clone()),
        };

        let mut response = self.client
            .get("https://neocities.org/api/info")
            .header(UserAgent::new(USER_AGENT))
            .header(Authorization(credentials))
            .send()
            .expect("Failed to send request");

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
        use reqwest::header::{Authorization, Basic, UserAgent};

        let credentials = Basic {
            username: self.username.clone(),
            password: Some(self.password.clone()),
        };

        let mut response = self.client
            .get("https://neocities.org/api/list")
            .header(UserAgent::new(USER_AGENT))
            .header(Authorization(credentials))
            .send()
            .expect("Failed to send request");

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

    pub fn upload(&self, name: String, path: PathBuf) -> Result<()> {
        trace!("Site::upload()");
        use reqwest::header::{Authorization, Basic, UserAgent};

        let credentials = Basic {
            username: self.username.clone(),
            password: Some(self.password.clone()),
        };

        let form = reqwest::multipart::Form::new()
            .file(name, path).unwrap();

        let mut response = self.client
            .post("https://neocities.org/api/upload")
            .header(UserAgent::new(USER_AGENT))
            .header(Authorization(credentials))
            .multipart(form)
            .send()
            .expect("Failed to send request");

        debug!("{:?}", response);

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
        use reqwest::header::{Authorization, Basic, UserAgent};

        let credentials = Basic {
            username: self.username.clone(),
            password: Some(self.password.clone()),
        };

        let mut query = String::new();

        for file in files {
            query.push_str("filenames[]=");
            query.push_str(&file);
            query.push('&');
        }

        let url = format!("https://neocities.org/api/delete?{}", query);

        let mut response = self.client
            .post(&url)
            .header(UserAgent::new(USER_AGENT))
            .header(Authorization(credentials))
            .send()
            .expect("Failed to send request");

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
