use serde_json as json;
use std::env;
use std::error;
use std::io::Write;
use std::process;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub mod cli;
use cli::Args;

pub type Result<T> = ::std::result::Result<T, Box<dyn error::Error>>;

fn main() {
    if let Err(err) = Args::parse(env::args_os()).and_then(try_main) {
        eprintln!("{}", err);
        process::exit(2);
    }
}

fn try_main(args: Args) -> Result<()> {
    match args.command {
        cli::Command::Request => run_request(args),
        cli::Command::PrintHelp => Ok(print_usage()),
        cli::Command::PrintVersion => Ok(print_version()),
    }
}

fn run_request(args: Args) -> Result<()> {
    let url = match reqwest::Url::parse(&args.url) {
        Ok(u) => u,
        Err(e) => {
            eprintln!("Error parsing URL '{:}'", &args.url);
            return Err(Box::new(e));
        }
    };

    let client = reqwest::blocking::Client::new();
    let res = client
        .request(args.method, url)
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

    if let Some(content_type) =
        res.headers().get(reqwest::header::CONTENT_TYPE)
    {
        if content_type == "application/json" {
            let message = res.json::<json::Value>().unwrap();
            json::to_writer_pretty(&mut stdout, &message)?;
            println!("");
        } else {
            writeln!(&mut stdout, "{}", res.text().unwrap())?;
        }
    }

    return Ok(());
}

fn print_usage() {
    eprintln!("{}", "\
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
}

fn print_version() {
    eprintln!("{:}", env!("CARGO_PKG_VERSION"));
}
