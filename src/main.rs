extern crate neo;
extern crate reqwest;

fn main() {
    let site = neo::site::Site::new("psbarrett".to_string(),
                                    "T[x1O\"qnGpr\"a>wAw\"U_Wkziv@O'nc\
                                     rc'yu)S7Cq-PRogy?Z46JcGp!dZ,)WRYOc"
                                        .to_string(),
                                    None);

    let info = site.info().unwrap();
    println!("Site: {}", info.sitename);

    let files = site.list().unwrap();
    println!("Files:");
    for file in files {
        println!("  {}", file.path);
    }

    site.upload("test.html".to_owned(), "asdf".to_owned()).expect("upload failed");
    site.delete(vec!["neocities.png".to_owned()]).expect("delete failed");
}
