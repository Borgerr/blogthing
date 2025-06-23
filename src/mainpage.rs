use axum::http::StatusCode;
use filetime::FileTime;
use maud::{Markup, html};

use crate::{dir_path, get_post_title, header};

//use std::path::Path;
use std::{fs, path::PathBuf};

/// Routed to from `/`.
/// Collects all of the top headlines from markdown files in specified directory,
/// and places them in a single page of links to each.
pub async fn main_page() -> (StatusCode, Markup) {
    let mut markdown_files: Vec<PathBuf> = fs::read_dir(&dir_path())
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
                @for md in markdown_files {
                    @if let Some(entry) = mainpage_entry(md) {
                        (entry)
                    }
                }
            }
        },
    )
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
            a href=(html_name)
            { (title) }
        }
    })
}
