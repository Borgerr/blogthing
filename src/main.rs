use axum::{Router, routing::get};
use clap::Parser;
use maud::{DOCTYPE, Markup, html};

use std::{
    fs,
    io::{self, BufRead},
    path::PathBuf,
    sync::OnceLock,
};

mod mainpage;
use mainpage::main_page;
mod posts;
use posts::get_post;

#[derive(Parser, Debug)]
#[command(version("0.1.0"), about = "A webserver that converts local markdown files to served static HTML, ideal for low effort blogs.", long_about = None)]
struct Args {
    #[arg(
        help = "URL exposed to the internet. Used for generating redirection to the blog itself on the fly."
    )]
    external_addr: String,
    #[clap(
        long,
        short,
        help = "Directory to find Markdown files. Defaults to current working directory."
    )]
    markdown_dir: Option<String>, // XXX: maybe want a std::path::Path-like type
    // bad and naughty admins may abuse this
    #[clap(
        long,
        short,
        action,
        help = "Boolean flag indicating there's a style.css present to serve in the `markdown_dir`."
    )]
    with_css: bool,
    #[clap(
        long,
        short,
        help = "Internal address of the webserver. This is what the internal TCP listening socket opens on. Defaults to `external_addr` Depending on setup or virtualization options, may want to change."
    )]
    internal_addr: Option<String>,
}

pub(crate) static CMDLINE_ARGS: OnceLock<Args> = OnceLock::new();

#[tokio::main]
async fn main() {
    CMDLINE_ARGS
        .set(Args::parse())
        .expect("CMDLINE_ARGS couldn't be initialized");

    let app = Router::new()
        .route("/", get(main_page))
        .route("/{post}", get(get_post)); // IDEA: add subdirectories to "group" different blog
    // subjects, and route based on that

    let addr = match CMDLINE_ARGS.get() {
        Some(args) => match &args.internal_addr {
            Some(addr) => addr,
            None => &args.external_addr,
        },
        None => panic!("was unable to extract address from CMDLINE_ARGS"),
    };
    // EVERYTHING after should have CMDLINE_ARGS accessible
    // as it's readonly and has, by now, been read
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("was unable to bind to specified port");
    axum::serve(listener, app)
        .await
        .expect("something catastrophic happened while working");
}

/// Formats a header for a webpage given a title.
pub(crate) fn header(title: &str) -> Markup {
    // TODO: do we want custom headers?
    html! {
        (DOCTYPE)
        meta charset="utf-8";
        // TODO: add link to main page
        h1 { (title) }  // TODO: this kinda sucks, should remove
    }
}

/// Gets the title of a post.
/// Removes any (reasonable) leading markdown formatting syntax.
pub(crate) fn get_post_title(md: PathBuf) -> Option<String> {
    let file = fs::File::open(md).ok()?;
    let reader = io::BufReader::new(file);

    Some(
        reader
            .lines()
            .into_iter()
            .next()? // first line
            .ok()?
            .trim_start_matches("# ") // removes header formatting from markdown string
            // TODO: do we want to enforce a h1 header?
            .to_string(),
    )
}

pub(crate) fn dir_path() -> String {
    CMDLINE_ARGS
        .get()
        .unwrap()
        .markdown_dir
        .clone()
        .unwrap_or("./".to_string())
}

