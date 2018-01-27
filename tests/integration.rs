extern crate neo;
extern crate dotenv;

use dotenv::dotenv;
use std::env;

fn setup() {
    dotenv().ok();
}

#[test]
fn full(){
    setup();

    let site = neo::site::Site::new(env::var("TEST_SITE").expect("TEST_SITE"),
                                    env::var("TEST_PASSWORD").expect("TEST_PASSWORD"),
                                    None);

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
