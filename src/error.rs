use std::convert::From;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::io;

use docopt;
use hyper;
use ini::ini;
use rocket;
use url;

#[derive(Debug)]
pub enum FromError {
    Io(io::Error),
    Ini(ini::Error),
    Hyper(hyper::Error),
    Docopt(docopt::Error),
    Rocket(rocket::config::ConfigError),
    Url(url::ParseError),
}

#[derive(Debug)]
pub enum Error {
    FromError(FromError),
    FromErrorMessage(FromError, &'static str),
    General(&'static str),
}

impl Error {
    pub fn new(s: &'static str) -> Self {
        Error::General(s)
    }

    pub fn add_message(self, s: &'static str) -> Self {
        match self {
            Error::FromError(from_error) => Error::FromErrorMessage(from_error, s),
            _ => panic!("Adding a message to an error with a message"),
        }
    }
}

pub type ProgressResult<T> = Result<T, Error>;

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error { Error::FromError(FromError::Io(err)) }
} 
impl From<ini::Error> for Error {
    fn from(err: ini::Error) -> Error { Error::FromError(FromError::Ini(err)) }
} 
impl From<hyper::Error> for Error {
    fn from(err: hyper::Error) -> Error { Error::FromError(FromError::Hyper(err)) }
}
impl From<docopt::Error> for Error {
    fn from(err: docopt::Error) -> Error { Error::FromError(FromError::Docopt(err)) }
}
impl From<rocket::config::ConfigError> for Error {
    fn from(err: rocket::config::ConfigError) -> Error { Error::FromError(FromError::Rocket(err)) }
}
impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Error { Error::FromError(FromError::Url(err)) }
}

impl Display for FromError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            &FromError::Io(ref err) => write!(f, "{}", err),
            &FromError::Ini(ref err) => write!(f, "{}", err),
            &FromError::Hyper(ref err) => write!(f, "{}", err),
            &FromError::Docopt(ref err) => write!(f, "{}", err),
            &FromError::Rocket(ref err) => write!(f, "{:?}", err),
            &FromError::Url(ref err) => write!(f, "{}", err),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            &Error::FromError(ref from_error) => write!(f, "{}", from_error),
            &Error::FromErrorMessage(ref from_error, ref s) => write!(f, "{}: {}", s, from_error),
            &Error::General(ref s) => write!(f, "Custom Error: {}", s),
        }
    }
}

pub fn add_message<E>(s: &'static str) -> impl Fn(E) -> Error where Error: From<E> {
    move |err| Error::from(err).add_message(s)
}
