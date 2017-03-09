#![feature(plugin)]
#![feature(conservative_impl_trait)]
#![plugin(rocket_codegen)]

#[macro_use] extern crate rocket_contrib;
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
use rocket::response::NamedFile;
use rocket::config::{active, Config, Environment, Value};
use rocket_contrib::JSON;
use docopt::Docopt;
use ini::Ini;
// use regex::Regex;
// use rustc_serialize::json;
use walkdir::WalkDir;
use url::Url;

mod error;

use error::{Error, ProgressResult, add_message};

fn get_stec_root_from_config() -> ProgressResult<PathBuf> {
    let config = active().ok_or(Error::new("No Rocket.toml file"))?;
    for (key, value) in config.extras() {
        if key == "stec_root" {
            if let &Value::String(ref stec_root) = value {
                return Ok(PathBuf::from(stec_root.to_string()));
            } else {
                return Err(Error::new("stec_root not a string in config file"));
            }
        }
    }
    return Err(Error::new("No stec_root in Rocket.toml"));
}

fn get_propath_from_config() -> ProgressResult<Vec<PathBuf>> {
    let stec_root = get_stec_root_from_config()?;
    return get_propath(&stec_root);
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

fn get_progress_file_path(file_path: &Path, propath: &Vec<PathBuf>) -> ProgressResult<PathBuf> {
    for prefix_path in propath {
        let mut path = prefix_path.to_owned();
        path.push(file_path);
        if path.exists() {
            return Ok(path);
        }
    }
    return Err(Error::new(format!("The file '{}' does not exist", file_path.to_string_lossy())));
}

fn get_propath(root_path: &Path) -> ProgressResult<Vec<PathBuf>> {
    // Get the stec.ini file
    let mut stec_ini = PathBuf::from(root_path);
    stec_ini.push("stec.ini");
    let mut stec_ini = File::open(stec_ini.clone()).map_err(add_message(format!("Could not find {}", stec_ini.to_string_lossy())))?;

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
        .ok_or(Error::new("No PROPATH field"))
}

#[get("/file/<file_path..>")]
fn get_file(file_path: PathBuf) -> ProgressResult<NamedFile> {
    let propath = get_propath_from_config()?;
    let full_path = get_progress_file_path(&file_path, &propath)?.to_string_lossy().into_owned();
    NamedFile::open(full_path.clone()).map_err(add_message(format!("Could not open file '{}'", full_path)))
}

#[get("/find/<query>", format="application/json")]
fn find_file(query: String) -> ProgressResult<JSON<Vec<String>>> {
    let stec_root = get_stec_root_from_config()?;
    let mut results = vec!();
    for entry in WalkDir::new(&stec_root).into_iter().filter_map(|e| e.ok()) {
        let file_name = entry.file_name().to_string_lossy().into_owned();
        let maybeExtension = entry.path().extension();
        if let Some(extension) = maybeExtension {
            if entry.file_type().is_file() && extension != "r" && file_name.contains(&query) {
                let path = String::from(relative_path(entry.path(), &stec_root).to_str().unwrap());
                results.push(path);
            }
        }
    }
    println!("{}: {:?}", query, results);
    return Ok(JSON(results));
}

fn main() {
    Rocket::ignite().mount("/", routes![get_file, find_file]).launch();
}
