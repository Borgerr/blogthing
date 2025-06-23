use axum::{Router, extract::Path, http::StatusCode, response::Html, routing::get};
use clap::Parser;
use filetime::FileTime;
use maud::{DOCTYPE, html};

//use std::path::Path;
use std::{fs, path::PathBuf, sync::OnceLock};

type Response = Html<(StatusCode, String)>;

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

async fn main_page() -> Response {
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

    markdown_files.sort_by(|md1, md2| {
        let md1_meta = fs::metadata(md1).unwrap();
        let md1_modtime = FileTime::from_last_modification_time(&md1_meta);
        let md2_meta = fs::metadata(md2).unwrap();
        let md2_modtime = FileTime::from_last_modification_time(&md2_meta);
        md2_modtime.cmp(&md1_modtime)
    });

    let text = format!("{:?}", markdown_files);
    Html((StatusCode::OK, text))
}

async fn get_post(Path(requested): Path<String>) -> Response {
    todo!("implement")
}
