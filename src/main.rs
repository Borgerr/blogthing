use axum::{Router, extract::Path, http::StatusCode, routing::get};
use clap::Parser;
use filetime::FileTime;
use maud::{DOCTYPE, Markup, html};

//use std::path::Path;
use std::{
    fs,
    io::{self, BufRead},
    path::PathBuf,
    sync::OnceLock,
};

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

static CMDLINE_ARGS: OnceLock<Args> = OnceLock::new();

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
    /*
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("was unable ")
    */
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("was unable to bind to specified port");
    axum::serve(listener, app)
        .await
        .expect("something catastrophic happened while working");
}

/// Routed to from `/`.
/// Collects all of the top headlines from markdown files in specified directory,
/// and places them in a single page of links to each.
async fn main_page() -> (StatusCode, Markup) {
    let dir_path = CMDLINE_ARGS
        .get()
        .unwrap()
        .markdown_dir
        .clone()
        .unwrap_or("./".to_string());

    let mut markdown_files: Vec<PathBuf> = fs::read_dir(&dir_path)
        .unwrap()
        .map(|p| p.unwrap().path())
        .filter(|pstr| match pstr.extension() {
            Some(x) => x == "md",
            None => false,
        })
        .collect();

    // post-order corresponds to how long ago the file was modified
    // XXX: assumes recently modified blogs (not recently created) want to be towards the top again
    markdown_files.sort_by(|md1, md2| {
        let md1_meta = fs::metadata(md1).unwrap();
        let md1_modtime = FileTime::from_last_modification_time(&md1_meta);
        let md2_meta = fs::metadata(md2).unwrap();
        let md2_modtime = FileTime::from_last_modification_time(&md2_meta);
        md2_modtime.cmp(&md1_modtime)
    });

    (
        StatusCode::OK,
        html! {
            (header("blog"))    // TODO: allow config?
            ul {
                @for md in markdown_files { // TODO: we definitely want to encapsulate in
                                            // another function. This will get very big.
                    @if let Some(entry) = mainpage_entry(md) {
                        (entry)
                    }
                }
            }
        },
    )
}

/// Formats a header for a webpage given a title.
fn header(title: &str) -> Markup {
    html! {
        (DOCTYPE)
        meta charset="utf-8";
        // TODO: add link to main page
        h1 { (title) }  // TODO: possibly want to allow this to be different
    }
}

/// Formats an entry (as a PathBuf) on the main blog page.
/// Currently assumes first line will always be encoded with a `#` at the start
/// for a h1 heading in Markdown.
fn mainpage_entry(md: PathBuf) -> Option<Markup> {
    let title = get_post_title(md.clone())?;
    Some(html! {
        @let html_name = md
            .with_extension("html")
            .file_name()
            .unwrap()
            .display()
            .to_string();
        li {
            a href=(format!("{}/{}",
                        CMDLINE_ARGS.get().unwrap().external_addr,
                        html_name
                    ))
            { (title) }
        }
    })
}

/// Gets the title of a post.
/// Currently assumes first line of the post will always be encoded with a `#` at the start
/// for a h1 heading in Markdown.
fn get_post_title(md: PathBuf) -> Option<String> {
    let file = fs::File::open(md).ok()?;
    let reader = io::BufReader::new(file);

    Some(
        reader
            .lines()
            .into_iter()
            .next()? // first line
            .ok()?
            .trim_start_matches("# ") // removes header formatting from markdown string
            .to_string(),
    )
}

async fn get_post(Path(requested): Path<String>) -> (StatusCode, Markup) {
    todo!("implement")
}
