#[macro_use]
extern crate nickel;
extern crate hyper;
extern crate rustc_serialize;
extern crate walkdir;
extern crate regex;
extern crate ini;

use nickel::{Nickel, HttpRouter, Mountable};
use std::path::Path;
use rustc_serialize::json;
use walkdir::WalkDir;
use regex::Regex;
use ini::Ini;

const PATH: &'static str = "C:/stec82";

fn get_propath() -> Result<Vec<String>, Error> {
    let mut stec_ini = try!(get_file(concat!(PATH, "/stec.ini")));
    stec_ini = stec_ini.replace("\\", "/");
    
    let conf = try!(Ini::load_from_str(&stec_ini).map_err(|err| Error::Ini("Could not parse ini file", err)));

    conf.section(Some("Startup"))
        .and_then(|section| section.get("PROPATH"))
        .map(|s| s.split(",").map(|s| String::from(s)).collect())
        .ok_or(Error::General("No PROPATH field"))
}

fn main() {
    let mut server = Nickel::new();

    server.mount("/file/", middleware! { |req, res| {
        // let mut path = String::from(PATH);
        // path.push_str(&req.origin.uri.to_string());

        println!("Getting file: {}", &req.origin.uri.to_string()[1..]);
        return res.send_file(Path::new(&req.origin.uri.to_string()[1..]));
        //let r_file = File::open(path);

        /*
        r_file.map(|file| {
            let mut body = String::new();
            file.read_to_string(&mut body);
            res.send(body)
            res.send_file(path);
        }).unwrap_or_else(|_| {
            res.set(StatusCode::NotFound);
            res.end()
        })
        */
    }});
    // server.mount("/file/", StaticFilesHandler::new("C:/stec82"));

    let find_regex = Regex::new("/find/(?P<contents>.+)").unwrap();
    server.get(find_regex, middleware! { |req, res| {
        let contents : &str = &req.param("contents").unwrap().replace("%2F", "/");
        let mut results = vec!();
        for entry in WalkDir::new(PATH).into_iter().filter_map(|e| e.ok()) {
            // println!("File: {:?}, {:?}", entry, entry.file_name());
            if entry.file_type().is_file() && (entry.file_name() == contents) {
                results.push(String::from(entry.path().to_str().unwrap()));
            }
        }
        return res.send(json::encode(&results).unwrap());
    }});

    server.listen("192.168.221.80:3000");
}
