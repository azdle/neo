extern crate neo;
extern crate dotenv;

use dotenv::dotenv;
use std::env;

fn setup() -> neo::Site {
    dotenv().ok();

    neo::Site::new(env::var("TEST_SITE").expect("TEST_SITE"),
                   env::var("TEST_PASSWORD").expect("TEST_PASSWORD"),
                   None)
}

#[test]
fn full(){
    let site = setup();

    let info = site.info().expect("info falied");
    println!("Site: {}", info.sitename);

    let files = site.list().expect("list failed");
    println!("Files:");
    for file in files {
        println!("  {}", file.path);
    }

    site.upload("test.html".to_owned(), "tests/assets/testfile.html".into()).expect("upload failed");
    site.delete(vec!["test.html".to_owned()]).expect("delete failed");
}

#[test]
fn upload_site_info(){
    let site = setup();

    site.upload("index.html".to_owned(), "tests/assets/siteinfo.html".into()).expect("upload failed");
}
