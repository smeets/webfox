use std::env;
// use std::cmp;
use std::ffi::{OsString};
// use std::fs;
use std::io::{self, Write};
// use std::path::{Path, PathBuf};
use std::process;
// use std::sync::Arc;
// use std::time::SystemTime;
use std::error;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use serde_json as json;
use clap;

type Result<T> = ::std::result::Result<T, Box<dyn error::Error>>;

pub mod app;

fn main() {
    if let Err(err) = clap_matches(env::args_os()).and_then(try_main) {
        eprintln!("{}", err);
        process::exit(2);
    }
}

fn try_main(_args: clap::ArgMatches<'static>) -> Result<()> {
    let client = reqwest::blocking::Client::new();
    let res = client.post("http://httpbin.org/post")
        .body("the exact body that is sent")
        .send()?;

    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    println!("{:?} {:?}", res.version(), res.status());
    for (key, val) in res.headers().iter() {
        write!(&mut stdout, "{}: ", key)?;
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
        write!(&mut stdout, "{}\n", val.to_str().unwrap())?;
        stdout.reset()?;
    }

    println!("");

    if let Some(content_type) = res.headers().get(reqwest::header::CONTENT_TYPE) {
        if content_type == "application/json" {
            let message = res.json::<json::Value>().unwrap();
            json::to_writer_pretty(&mut stdout, &message)?;
            println!("");
        } else {
            write!(&mut stdout, "{}", res.text().unwrap())?;
        }
    }
    process::exit(0)
}


fn clap_matches<I, T>(args: I) -> Result<clap::ArgMatches<'static>>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let err = match app::app().get_matches_from_safe(args) {
        Ok(matches) => return Ok(matches),
        Err(err) => err,
    };
    if err.use_stderr() {
        return Err(err.into());
    }
    // Explicitly ignore any error returned by write!. The most likely error
    // at this point is a broken pipe error, in which case, we want to ignore
    // it and exit quietly.
    //
    // (This is the point of this helper function. clap's functionality for
    // doing this will panic on a broken pipe error.)
    let _ = write!(io::stdout(), "{}", err);
    process::exit(0);
}
