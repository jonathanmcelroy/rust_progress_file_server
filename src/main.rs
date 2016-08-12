#[macro_use]
extern crate nickel;
extern crate hyper;
extern crate rustc_serialize;
extern crate walkdir;
extern crate regex;
extern crate ini;
extern crate docopt;

use nickel::{Nickel, HttpRouter, Mountable};
use std::path::{Path, PathBuf};
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

fn get_progress_file_path(file_path: &Path, propath: &Vec<PathBuf>) -> Result<PathBuf, Error> {
    for prefix_path in propath {
        let mut path = prefix_path.to_owned();
        path.push(file_path);
        println!("{:?}: {}", path, path.exists());
        if path.exists() {
            return Ok(path);
        }
    }
    return Err(Error::General("File does not exist"));
}

fn get_propath(root_path: &Path) -> Result<Vec<PathBuf>, Error> {
    let mut stec_ini = PathBuf::from(root_path);
    stec_ini.push("stec.ini");
    let mut stec_ini = try!(File::open(stec_ini).map_err(|err| Error::Io(err)));

    let mut stec_ini_contents = String::new();
    stec_ini.read_to_string(&mut stec_ini_contents);
    stec_ini_contents = stec_ini_contents.replace("\\", "/");

    let conf = try!(Ini::load_from_str(&stec_ini_contents).map_err(|err| Error::Ini("Could not parse ini file", err)));

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
    let args = Docopt::new(USAGE).unwrap().parse().unwrap();
    let root_path = PathBuf::from(args.get_str("<path>"));
    let propath = get_propath(&root_path).unwrap();

    let mut server = Nickel::new();

    // Get the file in the propath
    let file_regex = Regex::new("/file/(?P<file>.+)").unwrap();
    server.get(file_regex, middleware! { |req, res| {
        let file_path = PathBuf::from(req.param("file").unwrap()
                                      .replace("%2F", "/")
                                      .replace("%5C", "/"));
        let path = get_progress_file_path(&file_path, &propath).unwrap();
        return res.send_file(path);

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
