
use clap::{self, crate_authors, crate_version, App, AppSettings};
// use lazy_static::lazy_static;

const ABOUT: &str = "\
webfox (wx) is a modern HTTP/1 and HTTP/2 client in the spirit of httpie.

Project home page: https://github.com/smeets/webfox

Use -h for short descriptions and --help for more details.";

const USAGE: &str = "
    wx [OPTIONS] URL [HEADERS | QUERY | DATA ...]";

const TEMPLATE: &str = "\
{bin} ({version}): {about}
{author}

USAGE:{usage}

ARGS:
{positionals}

OPTIONS:
{unified}";

/// The http method.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HttpMethod {
    /// HTTP GET
    Get,
    /// HTTP HEAD
    Head,
    /// HTTP PUT
    Put,
    /// HTTP POST
    Post,
    /// HTTP DELETE
    Delete,
}

pub fn app() -> App<'static, 'static> {
    let mut app = App::new("wx")
        .author(crate_authors!())
        .version(crate_version!())
        // .long_version(LONG_VERSION.as_str())
        .about(ABOUT)
        .max_term_width(100)
        .setting(AppSettings::UnifiedHelpMessage)
        .setting(AppSettings::AllArgsOverrideSelf)
        .usage(USAGE)
        .template(TEMPLATE)
        .help_message("Prints help information. Use --help for more details.");
    // for arg in all_args_and_flags() {
    //     app = app.arg(arg.claparg);
    // }
    app
}
