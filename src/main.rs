#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate hyper;
extern crate rustc_serialize;
extern crate walkdir;
extern crate regex;
extern crate ini;
extern crate docopt;
extern crate url;

use std::fs::File;
use std::io::{Read, Write, stderr};
use std::path::{Path, PathBuf};
use std::process::exit;

use rocket::Rocket;
use rocket::config::{active, Config, Environment, Value};
use docopt::Docopt;
use ini::Ini;
// use regex::Regex;
// use rustc_serialize::json;
// use walkdir::WalkDir;
use url::Url;

mod error;

use error::{Error, ProgressResult};

fn get_propath_from_config() -> ProgressResult<Vec<PathBuf>> {
    let config = active().ok_or(Error::General("No config file"))?;
    for (key, value) in config.extras() {
        if key == "stec_root" {
            if let &Value::String(ref stec_root) = value {
                return get_propath(Path::new(stec_root));
            } else {
                return Err(Error::General("stec_root not a string in config file"));
            }
        }
    }
    return Err(Error::General("No stec_root in config file"));
}

/*
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
*/

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
    // Get the stec.ini file
    let mut stec_ini = PathBuf::from(root_path);
    stec_ini.push("stec.ini");
    let mut stec_ini = File::open(stec_ini)?;

    // Try to read the contents
    let mut stec_ini_contents = String::new();
    let _ = stec_ini.read_to_string(&mut stec_ini_contents);
    stec_ini_contents = stec_ini_contents.replace("\\", "/");

    // Parse the contents
    let conf = Ini::load_from_str(&stec_ini_contents)?;

    // Get the propath
    conf.section(Some("Startup"))
        .and_then(|section| section.get("PROPATH"))
        .map(|s| s.split(",").map(|s| {
            let mut path = PathBuf::from(root_path);
            path.push(s);
            return path;
        }).collect())
        .ok_or(Error::General("No PROPATH field"))
}

#[get("/file/<file_path..>")]
fn get_file(file_path: PathBuf) -> ProgressResult<String> {
    let propath = get_propath_from_config()?;
    let full_path = get_progress_file_path(&file_path, &propath)?;
    Ok(full_path.to_string_lossy().into_owned())
}

fn run() -> ProgressResult<()> {
    // Get the file in the propath
    //let file_regex = Regex::new("/file/(?P<file>.+)").unwrap();
    //server.get(file_regex, middleware! { |req, res| { "testing" }});
        /*let file_path = PathBuf::from(req.param("file").unwrap()
                                      .replace("%2F", "/")
                                      .replace("%5C", "/"));
        return match get_progress_file_path(&file_path, &propath) {
            Ok(path) => res.send_file(path),
            Err(err) => res.error(StatusCode::NotFound, "File not found")
        }
    }});
    */

    /*
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
    */

    Rocket::ignite().mount("/", routes![get_file]).launch();
    Ok(())
}

fn main() {
    let result = run();
    if let Err(err) = result {
        let _ = writeln!(&mut stderr(), "{}", err);
        exit(1);
    }
}
