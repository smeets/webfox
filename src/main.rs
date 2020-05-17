use std::env;
// use std::cmp;
use std::ffi::{OsString};
// use std::fs;
use std::io::{Write};
// use std::path::{Path, PathBuf};
use std::process;
// use std::sync::Arc;
// use std::time::SystemTime;
use std::error;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use serde_json as json;

use std::vec::{Vec};

type Result<T> = ::std::result::Result<T, Box<dyn error::Error>>;

pub enum ContentType {
    /// application/x-www-form-urlencoded
    Form,
    /// application/json
    Json,
    /// multipart/form-data
    Multipart
}

pub struct Args {
    method: reqwest::Method,
    url: String,
    header: reqwest::header::HeaderMap,
    query: Vec<(String, String)>,
    data: Vec<(String, String)>,
    format: ContentType,
}

impl Args {
    pub fn new() -> Self {
        Self{
            method: reqwest::Method::GET,
            url: String::with_capacity(100),
            header: reqwest::header::HeaderMap::new(),
            query: Vec::new(),
            data: Vec::new(),
            format: ContentType::Json,
        }
    }

    fn parse<I, T>(args: I) -> Result<Self>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let mut parsed_args = Self::new();

        let mut parsed_url = false;
        let mut parsed_method = false;

        let mut iter = args.into_iter().enumerate();
        while let Some((i, k)) = iter.next() {
            if i == 0 { continue; }

            let arg = k.clone().into();
            let argv = arg.to_str();

            // first, attempt to parse flags
            if let Some(text) = argv {
                if text.starts_with("-") {
                    match text {
                        "-h" | "--help" => print_usage(),
                        "-v" | "--version" => print_version(),
                        "-f" | "--form" => parsed_args.format = ContentType::Form,
                        "-m" | "--multi" => parsed_args.format = ContentType::Multipart,
                        "-d" | "--debug" => { /* show requests as well */ },
                        "-t" | "--time" => { /* show wall time of req+res */ },
                        _ => {
                            eprintln!("unknown option: {:}", text);
                            process::exit(1);
                        }
                    }
                    continue;
                }
            }

            // optionally try to parse a method
            if !parsed_method {
                let method = match argv {
                    Some("GET") => Ok(reqwest::Method::GET),
                    Some("PUT") => Ok(reqwest::Method::PUT),
                    Some("HEAD") => Ok(reqwest::Method::HEAD),
                    Some("POST") => Ok(reqwest::Method::POST),
                    Some("PATCH") => Ok(reqwest::Method::PATCH),
                    Some("OPTIONS") => Ok(reqwest::Method::OPTIONS),
                    Some("DELETE") => Ok(reqwest::Method::DELETE),
                    Some("TRACE") => Ok(reqwest::Method::TRACE),
                    Some("CONNECT") => Ok(reqwest::Method::CONNECT),
                    Some(_) => Err(()),
                    None => Err(())
                };

                if let Ok(m) = method {
                    parsed_args.method = m;
                    parsed_method = true;
                    continue;
                }
            }

            // must take url if not set yet
            if !parsed_url {
                let url = argv.unwrap().to_string();

                if url.starts_with(":/") {
                    // :/feta --> http://localhost/feta
                    parsed_args.url.push_str("http://localhost");
                    parsed_args.url.push_str(&url[1..]);
                } else if url.starts_with(":") {
                    // :3000/feta --> http://localhost:3000/feta
                    parsed_args.url.push_str("http://localhost");
                    parsed_args.url.push_str(&url);
                } else if !url.starts_with("http") {
                    // google.com/feta --> http://google.com/feta
                    parsed_args.url.push_str("http://");
                    parsed_args.url.push_str(&url);
                } else {
                    parsed_args.url.push_str(&url);
                }

                parsed_url = true;
                continue
            }

            // otherwise, try to parse request items

            // finally, if we didn't parse anything yet this is not a valid arg
            eprintln!("invalid argument: {:}", argv.unwrap());
            process::exit(1);
        }

        return Ok(parsed_args);
    }
}

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

fn print_usage() {
    eprintln!("\
USAGE: wx [FLAGS] [METHOD] URL [PARAM [PARAM ...]]

webfox (wx) is a json-default, simple HTTP client in the spirit of httpie.

FLAGS:
    -f, --form        Encode body as application/x-www-form-urlencoded
    -d, --debug       Print request headers and body
    -h, --help        Print this help message
    -v, --version     Print version information

METHOD:
    The case sensitive HTTP method to be used for the request, allowed:

    GET, POST, PUT, HEAD, PATCH, DELETE, CONNECT, TRACE

    Defaults to POST if any data is to be sent, otherwise GET is used.

URL:

PARAM:
    Optional key-value pairs specifying a http header, query string or data
    to be included in the request. Can be specified in any order.

    name:value - HTTP Header

        Host:google.com X-Api-Key:notverysecure

    name=value - Body data (string)

        first=john last=doe name=\"john doe\"

        form --> first=john&last=doe&name=john%20doe
        json --> {{
            \"first\": \"john\",
            \"last\": \"doe\",
            \"name\": \"john doe\"
        }}

    name:=value - JSON body data

        values:='[1,2,3]' quit:=true tree:='{{\"name\":\"root\", \"children\":[]}}'

        json --> {{
            \"values\": [1,2,3],
            \"quit\": true,
            \"tree\": {{
                \"name\": \"root\",
                \"children\": []
            }}
        }}


https://github.com/smeets/webfox
Axel Smeets <murlocbrand@gmail.com>");

    process::exit(0);
}

fn print_version() {
    eprintln!("{:}", env!("CARGO_PKG_VERSION"));
    process::exit(0);
}
