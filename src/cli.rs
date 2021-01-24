use std::ffi::OsString;
use std::vec::Vec;

use crate::Result;

#[derive(Debug, PartialEq)]
pub enum Command {
    /// Run query and exit
    Request,
    /// Print help / usage and exit
    PrintHelp,
    /// Print version information and exit
    PrintVersion,
}

/// Content type of the sent request
#[derive(Debug, PartialEq)]
pub enum ContentType {
    /// application/x-www-form-urlencoded
    Form,
    /// application/json
    Json,
    /// multipart/form-data
    Multipart,
}

/// >X
#[derive(Debug)]
struct ParseError {
    ctx: String,
}

impl ParseError {
    fn new(ctx: String) -> ParseError {
        return ParseError { ctx: ctx };
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.ctx)
    }
}

impl std::error::Error for ParseError {
    fn description(&self) -> &str {
        self.ctx.as_str()
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

#[derive(Debug, PartialEq)]
pub enum RequestItemType {
    Header,
    KeyVal,
    RawJson,
    Query,
}

pub struct Args {
    pub command: Command,
    pub method: reqwest::Method,
    pub url: String,
    pub headers: reqwest::header::HeaderMap,
    pub query: Vec<(String, String)>,
    pub data: Vec<(RequestItemType, String, String)>,
    pub format: ContentType,
}

struct Arg {
    short: char,
    long: &'static str,
    cb: fn(&mut Args),
}

impl Args {
    pub fn new() -> Self {
        Self {
            command: Command::Request,
            method: reqwest::Method::GET,
            url: String::with_capacity(100),
            headers: reqwest::header::HeaderMap::with_capacity(5),
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
            Arg {
                short: 'h',
                long: "help",
                cb: |args: &mut Args| args.command = Command::PrintHelp,
            },
            Arg {
                short: 'v',
                long: "version",
                cb: |args: &mut Args| args.command = Command::PrintVersion,
            },
            Arg {
                short: 'f',
                long: "form",
                cb: |args: &mut Args| args.format = ContentType::Form,
            },
            Arg {
                short: 'm',
                long: "multi",
                cb: |args: &mut Args| args.format = ContentType::Multipart,
            },
            Arg { short: 'd', long: "debug", cb: |_args: &mut Args| {} },
            Arg { short: 'm', long: "multi", cb: |_args: &mut Args| {} },
        ];

        let mut iter = args.into_iter().enumerate();
        while let Some((i, k)) = iter.next() {
            if i == 0 {
                continue;
            }

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
                        return Err(Box::new(ParseError::new(format!(
                            "unknown option: {}",
                            text
                        ))));
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
                            return Err(Box::new(ParseError::new(format!(
                                "unknown option: {}",
                                copt
                            ))));
                        }
                    }
                    continue;
                }
            }

            // optionally try to parse a method
            if !parsed_method {
                let method = match argv {
                    Some("GET") => Some(reqwest::Method::GET),
                    Some("PUT") => Some(reqwest::Method::PUT),
                    Some("HEAD") => Some(reqwest::Method::HEAD),
                    Some("POST") => Some(reqwest::Method::POST),
                    Some("PATCH") => Some(reqwest::Method::PATCH),
                    Some("OPTIONS") => Some(reqwest::Method::OPTIONS),
                    Some("DELETE") => Some(reqwest::Method::DELETE),
                    Some("TRACE") => Some(reqwest::Method::TRACE),
                    Some("CONNECT") => Some(reqwest::Method::CONNECT),
                    _ => None,
                };

                if let Some(m) = method {
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
                continue;
            }

            // otherwise, try to parse request items
            if let Some((typ, key, val)) = parse_var(argv.unwrap().to_string()) {
                match typ {
                    RequestItemType::Header => {
                        let header =
                            reqwest::header::HeaderName::from_bytes(key.as_bytes())?;
                        parsed_args.headers.insert(header, val.parse()?);
                    }
                    RequestItemType::KeyVal | RequestItemType::RawJson => {
                        parsed_args.data.push((typ, key, val));
                    }
                    RequestItemType::Query => {}
                }
                continue;
            }

            // finally, if we didn't parse anything yet this is not a valid arg
            return Err(Box::new(ParseError::new(format!(
                "invalid argument: {:}",
                argv.unwrap()
            ))));
        }

        // Catch missing url early, otherwise we'll show user an error from reqwest
        // "relative URL without base"
        if parsed_args.url.len() == 0
            && parsed_args.command != Command::PrintHelp
            && parsed_args.command != Command::PrintVersion
        {
            return Err(Box::new(ParseError::new(
                "missing url, run with -h to get help".to_string(),
            )));
        }

        return Ok(parsed_args);
    }
}

fn parse_var(arg: String) -> Option<(RequestItemType, String, String)> {
    let mut iter = arg.char_indices().peekable();
    while let Some((i, c)) = iter.next() {
        let typ = match c {
            //  Why tf does _ not work ?
            ':' if Some(&(i + 1, '=')) == iter.peek() => Some(RequestItemType::RawJson),
            ':' => Some(RequestItemType::Header),
            '=' if Some(&(i + 1, '=')) == iter.peek() => Some(RequestItemType::Query),
            '=' => Some(RequestItemType::KeyVal),
            _ => None,
        };

        if let Some(itt) = typ {
            let (key, val) = match itt {
                RequestItemType::RawJson | RequestItemType::Query => {
                    (arg[0..i].to_string(), arg[i + 2..].to_string())
                }
                RequestItemType::KeyVal | RequestItemType::Header => {
                    (arg[0..i].to_string(), arg[i + 1..].to_string())
                }
            };
            return Some((itt, key, val));
        }
    }
    return None;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn needurl() {
        Args::parse(vec!["program", "someurl.com"]).unwrap();
        assert!(Args::parse(vec!["program"]).is_err());
    }

    #[test]
    fn longopts() {
        let args = Args::parse(vec!["program", "someurl.com", "--form"]).unwrap();
        assert_eq!(args.format, ContentType::Form);
    }

    #[test]
    fn shortops() {
        let args = Args::parse(vec!["program", "somurl.com", "-fm"]).unwrap();
        assert_eq!(args.format, ContentType::Multipart);

        let args = Args::parse(vec!["program", "somurl.com", "-mf"]).unwrap();
        assert_eq!(args.format, ContentType::Form);
    }

    #[test]
    fn method() {
        // implicit GET
        let args = Args::parse(vec!["program", "someurl.com"]).unwrap();
        assert_eq!(args.method, reqwest::Method::GET);

        let methods = vec![
            ("GET", reqwest::Method::GET),
            ("POST", reqwest::Method::POST),
            ("PUT", reqwest::Method::PUT),
            ("PATCH", reqwest::Method::PATCH),
            ("HEAD", reqwest::Method::HEAD),
            ("OPTIONS", reqwest::Method::OPTIONS),
            ("CONNECT", reqwest::Method::CONNECT),
            ("TRACE", reqwest::Method::TRACE),
        ];
        // explicit method
        for (name, method) in methods {
            let args = Args::parse(vec!["program", name, "someurl.com"]).unwrap();
            assert_eq!(args.method, method);
        }
    }

    #[test]
    fn url_rules() {
        // implicit localhost
        let args = Args::parse(vec!["program", ":/hej"]).unwrap();
        assert_eq!(args.url, "http://localhost/hej");

        // implicit localhost with port
        let args = Args::parse(vec!["program", ":3000/hej"]).unwrap();
        assert_eq!(args.url, "http://localhost:3000/hej");

        // implicit http
        let args = Args::parse(vec!["program", "somesite.com:3000/hej"]).unwrap();
        assert_eq!(args.url, "http://somesite.com:3000/hej");
    }

    #[test]
    fn request_items() {
        assert!(
            Some((
                RequestItemType::Header,
                String::from("x-api-key"),
                String::from("1")
            )) == parse_var(String::from("x-api-key:1"))
        );

        assert!(
            Some((
                RequestItemType::KeyVal,
                String::from("x-api-key"),
                String::from("1")
            )) == parse_var(String::from("x-api-key=1"))
        );

        assert!(
            Some((
                RequestItemType::RawJson,
                String::from("x-api-key"),
                String::from("1")
            )) == parse_var(String::from("x-api-key:=1"))
        );

        assert!(
            Some((RequestItemType::Query, String::from("search"), String::from("1")))
                == parse_var(String::from("search==1"))
        );
    }
}
