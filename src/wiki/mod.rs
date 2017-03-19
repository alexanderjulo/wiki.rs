extern crate frontmatter;
extern crate hoedown;
extern crate walkdir;
extern crate yaml_rust;


use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::str;

use self::hoedown::{Markdown, Render};
use self::hoedown::renderer::html::{self, Html};
use self::walkdir::WalkDir;
use self::yaml_rust::yaml::Yaml;


fn convert_path_to_url(base_path: &str, path: &str) -> String {
    let url = String::from(path);
    let url = url.trim_left_matches(base_path);
    let url = url.trim_right_matches(".md");

    String::from(url)
}

fn convert_url_to_path(base_path: &str, url: &str) -> String {
    let mut path = String::new();
    path.push_str(base_path);
    path.push_str(url);
    path.push_str(".md");

    path
}

/// A single page within the wiki, which is backed by a markdown file
/// on disk
pub struct Page {
    /// the root path of the wiki this page is a part of
    pub base_path: PathBuf,
    /// the path to the page
    pub path: PathBuf,
    /// the url to this page, which is essentially the relative path
    /// of the file minus the file extension
    pub url: String,
    /// the raw body of the file, may be empty if the page has not been
    /// written to disk yet
    raw: String,
    /// the YAML frontmatter, might be empty
    pub meta: Option<Yaml>,
    /// the markdown body of the page, might be an empty string
    pub markdown: Markdown,
    /// the compiled HTML of the page
    pub html: String
}

impl Page {
    /// Creates a new `Page` object, tries to read the content of the backing
    /// file from the disk and interprets eventual data, including the
    /// frontmatter and HTML
    /// # Errors
    /// This will return an error if i.e. the reading of the file fails
    /// because of lacking permissions or non utf-8 content
    pub fn new(base_path: PathBuf, path: PathBuf) -> Result<Page, io::Error> {

        let url = convert_path_to_url(
            base_path.to_str().unwrap(),
            path.to_str().unwrap()
        );

        let mut page = Page{
            base_path: base_path,
            path: path,
            url: String::from(url),
            raw: String::from(""),
            meta: None,
            markdown: Markdown::new(""),
            html: String::from(""),
        };
        try!(page.read_from_file());
        page.load();
        Ok(page)
    }

    /// Reads the contents of the underlying files from the disk
    /// # Errors
    /// This will return an error if i.e. the reading of the file fails
    /// because of lacking permissions or non utf-8 content
    fn read_from_file(&mut self) -> Result<(), io::Error> {
        let mut f = try!(File::open(self.path.as_path()));
        let mut buffer = String::new();
        try!(f.read_to_string(&mut buffer));
        self.raw = buffer;
        Ok(())
    }

    /// Interprets the raw data, among other things loading the frontmatter
    /// and converting markdown to html.
    fn load(&mut self) {
        let mut html = Html::new(html::Flags::empty(), 0);
        match frontmatter::parse_and_find_content(self.raw.as_str()) {
            Ok((meta, markdown)) => {
                self.meta = meta;
                self.markdown = Markdown::new(markdown);
                self.html = String::from(
                    html.render(&self.markdown).to_str().unwrap()
                );
            }
            Err(_) => ()
        }
    }

    fn save_to_file(&self) -> Result<(), io::Error> {
        let mut f = try!(File::create(self.path.as_path()));
        try!(f.write_all(self.raw.as_bytes()));
        try!(f.sync_all());
        Ok(())
    }

}

/// A wiki object
pub struct Wiki {
    /// the root path of the wiki
    pub path: PathBuf,
    /// the pages that are contained in this wiki
    pub pages: Vec<Page>,
}

impl Wiki {
    /// Creates a new `Wiki` object. Will automatically load all pages
    /// that are contained
    pub fn new(pathname: &str) -> Wiki {
        let mut wiki = Wiki {
            path: Path::new(pathname).to_path_buf(),
            pages: Vec::new(),
        };
        wiki.load_pages();
        wiki
    }

    /// Load all the pages in the wiki
    fn load_pages(&mut self) {
        // make sure we do not duplicate shit by clearing
        // the vector first if necessary
        if !self.pages.is_empty() {
            self.pages.truncate(0);
        }

        for entry in WalkDir::new(self.path.clone()) {
            let entry = entry.unwrap();
            let entry = entry.path();
            let entry_path_str = entry.to_str().unwrap();
            if entry.is_file() && entry_path_str.ends_with(".md") {
                match Page::new(self.path.clone(), entry.to_path_buf()) {
                    Ok(page) => self.pages.push(page),
                    Err(e) => println!(
                        "Failed loading {}: {}",
                        entry_path_str,
                        e
                    )
                }
            }
        }
    }

    /// Will get an individual page object given a URL
    pub fn get(&self, url: &str) -> Option<&Page> {
        for page in self.pages.iter() {
            if page.url == url {
                return Some(page);
            }
        }
        None
    }

}


#[cfg(test)]
mod tests {
    #[test]
    fn test_convert_path_to_url() {
        assert_eq!(
            super::convert_path_to_url(
                "/wikidir",
                "/lol/what/a/path.md"
            ),
            "/lol/what/a/path"
        )
    }

    #[test]
    fn test_convert_url_to_path() {
        assert_eq!(
            super::convert_url_to_path(
                "/wikidir",
                "/lol/what/a/path",
            ),
            "/wikidir/lol/what/a/path.md"
        )
    }
}
