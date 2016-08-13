#[macro_use]
extern crate nickel;
extern crate hyper;
extern crate rustc_serialize;
extern crate walkdir;
extern crate regex;
extern crate ini;
extern crate docopt;

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use docopt::Docopt;
use ini::Ini;
use nickel::{Nickel, HttpRouter, Mountable};
use nickel::status::StatusCode;
use regex::Regex;
use rustc_serialize::json;
use walkdir::WalkDir;

mod error;

use error::{Error, ProgressResult, unwrap_or_exit};

fn relative_path(path: &Path, root_path: &Path) -> PathBuf {
    let mut result = PathBuf::new();

    let mut i_path = path.iter();
    let mut i_root = root_path.iter();
    loop {
        let e_path = i_path.next();
        let e_root = i_root.next();
        if e_path != e_root {
            result.push(e_path.unwrap());
            break;
        }
    }
    result.push(i_path);
    return result;
}

fn get_progress_file_path(file_path: &Path, propath: &Vec<PathBuf>) -> ProgressResult<PathBuf> {
    for prefix_path in propath {
        let mut path = prefix_path.to_owned();
        path.push(file_path);
        if path.exists() {
            return Ok(path);
        }
    }
    return Err(Error::General("File does not exist"));
}

fn get_propath(root_path: &Path) -> ProgressResult<Vec<PathBuf>> {
    let mut stec_ini = PathBuf::from(root_path);
    stec_ini.push("stec.ini");
    let mut stec_ini = try!(File::open(stec_ini));

    let mut stec_ini_contents = String::new();
    stec_ini.read_to_string(&mut stec_ini_contents);
    stec_ini_contents = stec_ini_contents.replace("\\", "/");

    let conf = try!(Ini::load_from_str(&stec_ini_contents));

    conf.section(Some("Startup"))
        .and_then(|section| section.get("PROPATH"))
        .map(|s| s.split(",").map(|s| {
            let mut path = PathBuf::from(root_path);
            path.push(s);
            return path;
        }).collect())
        .ok_or(Error::General("No PROPATH field"))
}

const USAGE: &'static str = "
Usage: main <ip> <path>
";

fn main() {
    let args = unwrap_or_exit(Docopt::new(USAGE).unwrap().parse());
    let root_path = PathBuf::from(args.get_str("<path>"));
    let propath = unwrap_or_exit(get_propath(&root_path));

    let mut server = Nickel::new();

    // Get the file in the propath
    let file_regex = Regex::new("/file/(?P<file>.+)").unwrap();
    server.get(file_regex, middleware! { |req, res| {
        let file_path = PathBuf::from(req.param("file").unwrap()
                                      .replace("%2F", "/")
                                      .replace("%5C", "/"));
        return match get_progress_file_path(&file_path, &propath) {
            Ok(path) => res.send_file(path),
            Err(err) => res.error(StatusCode::NotFound, "File not found")
        }
    }});

    // Find the file based upon its filename
    let find_regex = Regex::new("/find/(?P<contents>.+)").unwrap();
    server.get(find_regex, middleware! { |req, res| {
        let contents : &str = &req.param("contents").unwrap().replace("%2F", "/").replace("%5C", "/");
        let mut results = vec!();
        for entry in WalkDir::new(&root_path).into_iter().filter_map(|e| e.ok()) {
            let file_name : String = entry.file_name().to_string_lossy().into_owned();
            // println!("File: {:?}, {:?}", entry, entry.file_name());
            if entry.file_type().is_file() && file_name.contains(contents) {
                let path = String::from(relative_path(entry.path(), &root_path).to_str().unwrap());
                results.push(path);
            }
        }
        println!("{}: {:?}", contents, results);
        return res.send(json::encode(&results).unwrap());
    }});

    let ip = args.get_str("<ip>");
    server.listen(ip);
}
