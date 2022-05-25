use reqwest;
use std::path::PathBuf;

use errors::*;

const USER_AGENT: &'static str = concat!("neo/", env!("CARGO_PKG_VERSION"));

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
    //pub size: i64,
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
    client: reqwest::blocking::Client,
}

impl Site {
    pub fn new(auth: Auth) -> Site {
        trace!("Site::new()");
        let client = reqwest::blocking::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("build client with static user agent");

        Site { auth, client }
    }

    pub fn with_key(key: String) -> Site {
        trace!("Site::with_key()");
        let client = reqwest::blocking::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("build client with static user agent");
        Site {
            auth: Auth::Key(Key { key }),
            client,
        }
    }

    pub fn with_password(user: String, password: String) -> Site {
        trace!("Site::with_password()");
        let client = reqwest::blocking::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("build client with static user agent");

        Site {
            auth: Auth::Password(Password { user, password }),
            client,
        }
    }

    fn set_auth(
        &self,
        request: reqwest::blocking::RequestBuilder,
    ) -> reqwest::blocking::RequestBuilder {
        match self.auth {
            Auth::Key(ref key) => {
                debug!("auth with bearer token");
                request.bearer_auth(key.key.clone())
            }
            Auth::Password(ref password) => {
                debug!("auth with password");
                request.basic_auth(password.user.clone(), Some(password.password.clone()))
            }
        }
    }

    pub fn info(&self) -> Result<Info> {
        trace!("Site::info()");
        let request = self.client.get("https://neocities.org/api/info");
        let request = self.set_auth(request);

        debug!("request: {:?}", request);

        let response = request.send().expect("Failed to send request");

        debug!("response: {:?}", response);

        if response.status().is_success() {
            let r: InfoResult = response.json().unwrap();
            Ok(r.info)
        } else {
            let r: ::std::result::Result<ErrorResult, ::reqwest::Error> = response.json();
            match r {
                Ok(r) => Err(ErrorKind::ServerError(r).into()),
                Err(e) => Err(ErrorKind::UnexpectedResponse(e).into()),
            }
        }
    }

    pub fn list(&self) -> Result<Vec<File>> {
        trace!("Site::list()");
        let request = self.client.get("https://neocities.org/api/list");
        let request = self.set_auth(request);

        debug!("request: {:?}", request);

        let response = request.send().expect("Failed to send request");

        debug!("response: {:?}", response);

        if response.status().is_success() {
            let r: ListResult = response.json().unwrap();
            Ok(r.files)
        } else {
            let r: ::std::result::Result<ErrorResult, ::reqwest::Error> = response.json();
            match r {
                Ok(r) => Err(ErrorKind::ServerError(r).into()),
                Err(e) => Err(ErrorKind::UnexpectedResponse(e).into()),
            }
        }
    }

    pub fn upload(&self, path: String, file: PathBuf) -> Result<()> {
        use std::fs::File;
        use std::io::Read;
        trace!("Site::upload()");
        debug!("path: {}", path);
        debug!("file: {}", file.to_string_lossy());

        let mut file_contents = Vec::new();
        let mut file = File::open(file).unwrap();
        file.read_to_end(&mut file_contents)
            .expect("read file to upload");

        debug!("file contents length: {}", file_contents.len());

        let part = reqwest::blocking::multipart::Part::bytes(file_contents).file_name(path.clone());
        let form = reqwest::blocking::multipart::Form::new().part(path, part);

        debug!("form: {:?}", form);

        let url = "https://neocities.org/api/upload";

        let request = self.client.post(url);
        let request = self.set_auth(request);

        let request = request.multipart(form);

        debug!("request: {:?}", request);

        let response = request.send().expect("Failed to send request");

        debug!("response: {:?}", response);
        //debug!("response: {}", response.text().unwrap_or("Err".to_owned()));

        if response.status().is_success() {
            Ok(())
        } else {
            let r: ::std::result::Result<ErrorResult, ::reqwest::Error> = response.json();
            match r {
                Ok(r) => Err(ErrorKind::ServerError(r).into()),
                Err(e) => Err(ErrorKind::UnexpectedResponse(e).into()),
            }
        }
    }

    pub fn delete(&self, files: Vec<String>) -> Result<()> {
        trace!("Site::delete()");

        let mut query = String::new();

        for file in files {
            query.push_str("filenames[]=");
            query.push_str(&file);
            query.push('&');
        }

        let url = format!("https://neocities.org/api/delete?{}", query);

        let request = self.client.post(&url);
        let request = self.set_auth(request);

        debug!("request: {:?}", request);

        let response = request.send().expect("Failed to send request");

        debug!("response: {:?}", response);

        if response.status().is_success() {
            Ok(())
        } else {
            let r: ::std::result::Result<ErrorResult, ::reqwest::Error> = response.json();
            match r {
                Ok(r) => Err(ErrorKind::ServerError(r).into()),
                Err(e) => Err(ErrorKind::UnexpectedResponse(e).into()),
            }
        }
    }
}
