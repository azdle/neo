#![recursion_limit = "1024"]
#[macro_use]
extern crate error_chain;
extern crate neo;
extern crate reqwest;

// Note that this is different than the errors module in lib.rs
mod errors {
    error_chain!{}
}
use errors::*;

fn main() {
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

    site.upload("test.html".to_owned(), "./testfile.html".into()).expect("upload failed");
    site.delete(vec!["test.html".to_owned()]).expect("delete failed");

    Ok(())
}
