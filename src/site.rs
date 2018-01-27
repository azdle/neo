use reqwest;

#[derive(Serialize, Deserialize, Debug)]
struct InfoResult {
    result: String,
    info: Info,
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
        let client = reqwest::Client::new();

        Site {
            username: username,
            password: password,
            site: site,
            client: client,
        }
    }

    pub fn info(&self) -> Result<Info, String> {
        use reqwest::header::{Authorization, Basic, UserAgent};

        let credentials = Basic {
            username: self.username.clone(),
            password: Some(self.password.clone()),
        };

        let mut response = self.client
            .get("https://neocities.org/api/info")
            .header(UserAgent::new(format!("neo/{}", env!("CARGO_PKG_VERSION"))))
            .header(Authorization(credentials))
            .send()
            .expect("Failed to send request");

        if response.status().is_success() {
            let r: InfoResult = response.json().unwrap();
            Ok(r.info)
        } else {
            Err("bad status response".to_owned())
        }
    }

    pub fn list(&self) -> Result<Vec<File>, String> {
        use reqwest::header::{Authorization, Basic};

        let credentials = Basic {
            username: self.username.clone(),
            password: Some(self.password.clone()),
        };

        let mut response = self.client
            .get("https://neocities.org/api/list")
            .header(Authorization(credentials))
            .send()
            .expect("Failed to send request");

        if response.status().is_success() {
            let r: ListResult = response.json().unwrap();
            Ok(r.files)
        } else {
            Err("bad status response".to_owned())
        }
    }

    pub fn upload(&self, name: String, content: String) -> Result<(), String> {
        use reqwest::header::{Authorization, Basic};
        use std::io::Read;
        use std::collections::HashMap;

        let credentials = Basic {
            username: self.username.clone(),
            password: Some(self.password.clone()),
        };

        let mut body = HashMap::new();
        body.insert(name, content);

        let mut response = self.client
            .post("https://neocities.org/api/upload")
            .header(Authorization(credentials))
            .form(&body)
            .send()
            .expect("Failed to send request");

        println!("{:?}", response);

        if response.status().is_success() {
            Ok(())
        } else {
            let mut resp_body = String::new();
            response.read_to_string(&mut resp_body).unwrap();
            let error = format!("Bad Response on Upload: {:?}\n{}",
                                response.status(),
                                resp_body);
            Err(error)
        }
    }

    pub fn delete(&self, files: Vec<String>) -> Result<(), String> {
        use reqwest::header::{Authorization, Basic};

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

        let response = self.client
            .post(&url)
            .header(Authorization(credentials))
            .send()
            .expect("Failed to send request");

        if response.status().is_success() {
            Ok(())
        } else {
            let error = format!("Bad Response on Delete: {:?}", response.status());
            Err(error)
        }
    }
}
