use std::env;
// use std::cmp;
// use std::fs;
use std::io::{Write};
// use std::path::{Path, PathBuf};
use std::process;
// use std::sync::Arc;
// use std::time::SystemTime;
use std::error;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use serde_json as json;

pub mod cli;
use cli::{Args};

pub type Result<T> = ::std::result::Result<T, Box<dyn error::Error>>;


fn main() {
    if let Err(err) = Args::parse(env::args_os()).and_then(try_main) {
        eprintln!("{}", err);
        process::exit(2);
    }
}

fn try_main(args: Args) -> Result<()> {
    let url = match reqwest::Url::parse(&args.url) {
        Ok(u) => u,
        Err(e) => {
            eprintln!("Error parsing URL '{:}': {:?}", &args.url, e);
            process::exit(1);
        }
    };

    let client = reqwest::blocking::Client::new();
    let res = client.request(args.method, url)
        .body("the exact body that is sent")
        .send()?;

    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    println!("{:?} {:?}", res.version(), res.status());
    for (key, val) in res.headers().iter() {
        write!(&mut stdout, "{}: ", key)?;
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
        write!(&mut stdout, "{}\n", val.to_str()?)?;
        stdout.reset()?;
    }

    println!("");

    if let Some(content_type) = res.headers().get(reqwest::header::CONTENT_TYPE) {
        if content_type == "application/json" {
            let message = res.json::<json::Value>().unwrap();
            json::to_writer_pretty(&mut stdout, &message)?;
            println!("");
        } else {
            writeln!(&mut stdout, "{}", res.text().unwrap())?;
        }
    }
    process::exit(0)
}
