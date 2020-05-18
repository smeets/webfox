use std::process;
use std::vec::{Vec};
use std::ffi::{OsString};

use crate::{Result};

/// Content type of the sent request
pub enum ContentType {
    /// application/x-www-form-urlencoded
    Form,
    /// application/json
    Json,
    /// multipart/form-data
    Multipart
}

/// Parsed and collected arguments
pub struct Args {
    pub method: reqwest::Method,
    pub url: String,
    pub header: reqwest::header::HeaderMap,
    pub query: Vec<(String, String)>,
    pub data: Vec<(String, String)>,
    pub format: ContentType,
}

struct Arg {
    short: char,
    long: &'static str,
    cb: fn(&mut Args),
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

    pub fn parse<I, T>(args: I) -> Result<Self>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let mut parsed_args = Self::new();

        let mut parsed_url = false;
        let mut parsed_method = false;

        let options = vec![
            Arg{ short: 'h', long: "help",    cb: |_args: &mut Args| print_usage() },
            Arg{ short: 'v', long: "version", cb: |_args: &mut Args| print_version() },
            Arg{ short: 'f', long: "form",    cb: |args: &mut Args| args.format = ContentType::Form },
            Arg{ short: 'm', long: "multi",   cb: |args: &mut Args| args.format = ContentType::Multipart },
            Arg{ short: 'd', long: "debug",   cb: |_args: &mut Args| {} },
            Arg{ short: 'm', long: "multi",   cb: |_args: &mut Args| {} },
        ];

        let mut iter = args.into_iter().enumerate();
        while let Some((i, k)) = iter.next() {
            if i == 0 { continue; }

            let arg = k.clone().into();
            let argv = arg.to_str();

            // first, attempt to parse flags
            if let Some(text) = argv {
                // long options are given individually: --form --zen
                if text.starts_with("--") {
                    let mut found = false;
                    for arg in &options {
                        if arg.long == &text[2..] {
                            (arg.cb)(&mut parsed_args);
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        eprintln!("unknown option: {:}", text);
                        process::exit(1);
                    }
                    continue;
                } else if text.starts_with("-") {
                    // short options can be merged: -fz
                    for copt in text[1..].chars() {
                        let mut found = false;
                        for arg in &options {
                            if arg.short == copt {
                                (arg.cb)(&mut parsed_args);
                                found = true;
                                break;
                            }
                        }
                        if !found {
                            eprintln!("unknown option: {:}", copt);
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
