use serde_json as json;
use serde_json;
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
        eprintln!("{:}", err);
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
    let url = reqwest::Url::parse(&args.url)?;

    let req = reqwest::blocking::Client::new()
        .request(args.method, url)
        .query(&args.query)
        .headers(args.headers);

    let res = match args.format {
        cli::ContentType::Form => req.form(&build_form(&args.data)?),
        cli::ContentType::Json => req.json(&build_json(&args.data)?),
        cli::ContentType::Multipart => {
            req.multipart(reqwest::blocking::multipart::Form::new())
        }
    }
    .send()?;

    let mut stderr = StandardStream::stderr(ColorChoice::Always);
    eprintln!("{:?} {:?}", res.version(), res.status());
    for (key, val) in res.headers().iter() {
        write!(&mut stderr, "{}: ", key)?;
        stderr.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
        write!(&mut stderr, "{}\n", val.to_str()?)?;
        stderr.reset()?;
    }

    eprintln!("");

    let mut stdout = StandardStream::stdout(ColorChoice::Auto);


    match res.content_length() {
        Some(length) if length > 0 => match res.headers().get(reqwest::header::CONTENT_TYPE) {
            Some(content_type) if content_type.to_str()?.contains("application/json") => json::to_writer_pretty(&mut stdout, &res.json::<json::Value>()?)?,
            _ => stdout.write_all(res.text()?.as_bytes())?
        },
        Some(_) => {}, /* content_length == 0 */
        _ => stdout.write_all(res.text()?.as_bytes())?
    };

    return Ok(());
}

fn build_form(
    args: &Vec<(cli::RequestItemType, String, String)>,
) -> Result<std::collections::HashMap<&String, &String>> {
    let mut map = std::collections::HashMap::new();
    for (_, key, val) in args.iter() {
        map.insert(key, val);
    }
    Ok(map)
}

fn build_json(
    args: &Vec<(cli::RequestItemType, String, String)>,
) -> Result<serde_json::Value> {
    let mut map = serde_json::Map::new();
    for (typ, key, val) in args.iter() {
        let x = match typ {
            cli::RequestItemType::KeyVal => serde_json::Value::String(val.to_string()),
            cli::RequestItemType::RawJson => serde_json::from_str(val)
                .map_err(|err| format!("error parsing json: {}", err.to_string()))?,
            _ => unreachable!(),
        };
        map.insert(key.to_string(), x);
    }
    Ok(serde_json::Value::Object(map))
}

fn print_usage() {
    eprintln!(
        "{}",
        "\
usage: wx [FLAGS] [METHOD] URL [PARAM [PARAM ...]]

FLAGS:
    -f, --form        Encode body data as application/x-www-form-urlencoded
    -d, --debug       Print request headers and body
    -h, --help        Print this help message
    -v, --version     Print version information

METHOD:
    GET, POST, PUT, HEAD, PATCH, DELETE, CONNECT or TRACE
    Defaults to POST if body data PARAM exists, otherwise GET.

PARAM:
    HTTP Header:        name:value  (e.g. Host:google.com)
    Query string:       name==value (e.g. q==search)
    Body data (string): name=value  (e.g. first=john)
    Body data (json):   name:=value (e.g. values:=\"[1,2,3]\")

EXAMPLE:
    wx POST https://my.api.se/some X-API-KEY:badcat key=home count:=5
    wx https://google.com q==\"batman movies\"

https://github.com/smeets/webfox
Axel Smeets <murlocbrand@gmail.com>"
    );
}

fn print_version() {
    eprintln!("webfox {:}", env!("CARGO_PKG_VERSION"));
}
