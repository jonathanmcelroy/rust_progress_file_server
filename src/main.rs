#[macro_use]
extern crate nickel;
extern crate hyper;
extern crate rustc_serialize;
extern crate walkdir;
extern crate regex;
extern crate ini;
extern crate docopt;

use nickel::{Nickel, HttpRouter, Mountable};
use std::path::Path;
use rustc_serialize::json;
use walkdir::WalkDir;
use regex::Regex;
use ini::Ini;
use std::fs::File;
use std::io::Read;
use docopt::Docopt;

#[derive(Debug)]
enum Error {
    Io(std::io::Error),
    Ini(&'static str, ini::ini::Error),
    Hyper(hyper::Error),
    General(&'static str),
}

fn get_propath(root_path: &str) -> Result<Vec<String>, Error> {
    let mut path = String::from(root_path);
    path.push_str("/stec.ini");
    let mut stec_ini = try!(File::open(path).map_err(|err| Error::Io(err)));

    let mut stec_ini_contents = String::new();
    stec_ini.read_to_string(&mut stec_ini_contents);
    stec_ini_contents = stec_ini_contents.replace("\\", "/");

    let conf = try!(Ini::load_from_str(&stec_ini_contents).map_err(|err| Error::Ini("Could not parse ini file", err)));

    conf.section(Some("Startup"))
        .and_then(|section| section.get("PROPATH"))
        .map(|s| s.split(",").map(|s| String::from(s)).collect())
        .ok_or(Error::General("No PROPATH field"))
}

const USAGE: &'static str = "
Usage: main <ip> <path>
";

fn main() {
    let args = Docopt::new(USAGE).unwrap().parse().unwrap();
    let root_path = String::from(args.get_str("<path>"));
    let propath = get_propath(&root_path).unwrap();

    let mut server = Nickel::new();

    let file_regex = Regex::new("/file/(?P<file>.+)").unwrap();
    server.mount(file_regex, middleware! { |req, res| {
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
        for entry in WalkDir::new(&root_path).into_iter().filter_map(|e| e.ok()) {
            // println!("File: {:?}, {:?}", entry, entry.file_name());
            if entry.file_type().is_file() && (entry.file_name() == contents) {
                results.push(String::from(entry.path().to_str().unwrap()));
            }
        }
        return res.send(json::encode(&results).unwrap());
    }});

    let ip = args.get_str("<ip>");
    server.listen(ip);
}
