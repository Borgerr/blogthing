use axum::{extract::Path, http::StatusCode};
use maud::{Markup, html, PreEscaped};

use std::{
    fs,
    io::Read,
    path::PathBuf,
};

use crate::{dir_path, header};

pub async fn get_post(Path(requested): Path<String>) -> (StatusCode, Markup) {
    let notfound = (
        StatusCode::NOT_FOUND,
        html! { // TODO: obviously we need a better response here
            ("Guh!")
        },
    );

    // asserts requested file is html
    // likely won't want to change this. keep it simple
    match PathBuf::from(requested.clone()).extension() {
        Some(os_str) => {
            if os_str.to_str() != Some("html") {
                return notfound;
            }
        }
        None => return notfound,
    };

    let markdown_file_path =
        PathBuf::from(format!("{}{}", dir_path(), requested)).with_extension("md");

    match construct_post(markdown_file_path) {
        Some(mkp) => (StatusCode::OK, mkp),
        None => notfound,
    }
}

fn construct_post(md: PathBuf) -> Option<Markup> {
    if !fs::exists(&md).ok()? {
        return None;
    }
    //let title = get_post_title(md.clone())?;
    let mut file = fs::File::open(md).ok()?;
    let mut md_content = String::new();
    file.read_to_string(&mut md_content).ok()?;
    let html_content = markdown::to_html(&md_content);

    Some(html! {
        (header("dog wit a blog"))  // TODO: see comments in header function
        (PreEscaped(html_content))
    })
}
