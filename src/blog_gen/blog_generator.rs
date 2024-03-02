use std::fs::{self, File};
use std::io::prelude::*;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use glob::glob;
use pulldown_cmark::{html, Options, Parser};
use scraper::{Html, Selector};
use serde::Serialize;
use tera::Tera;
use thiserror::Error;

use super::html_template::{get_html_template, get_index_page_template};

#[derive(Error, Debug)]
pub enum BlogGeneratorError {
    #[error("the path ({0}) to css sources is invalid: {1}")]
    InvalidCSSPath(String, String),

    #[error("the css source file ({0}) could not be read: {1}")]
    CSSSourceError(String, String),

    #[error("the path to markdown sources dir ({0}) is invalid: {1}")]
    InvalidMarkDownPath(String, String),

    #[error("an error occurred reading a markdown source file {0}: {1}")]
    MarkDownFileError(String, String),

    #[error("an error occurred getting the metadata for a markdown source file {0}: {1}")]
    MarkDownMetadataError(String, String),

    #[error("the path ({0}) to rendered output directory is invalid: {1}")]
    InvalidRenderedOutputPath(String, String),

    #[error("an error occurred while attempting to write output file {0}: {1}")]
    FileWriteError(String, String),

    #[error("an error occurred while attempting to add a ({0}) template: {1}")]
    TemplateAddError(String, String),

    #[error("an error occurred while attempting to use a ({0}) template: {1}")]
    TemplateUseError(String, String),
}

#[derive(Clone, Debug, Default)]
struct MarkDownFile {
    file_name: PathBuf,
    file_path_buf: PathBuf,
    created_time: DateTime<Utc>,
    title_from_md: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize)]
struct Page {
    title: String,
    url: String,
}

pub struct BlogGenerator {
    base_url: String,
    css_source_file: String,
    markdown_sources_dir: String,
    rendered_outputs_dir: String,
}

impl BlogGenerator {
    pub fn new(
        base_url: String,
        css_source_file: String,
        markdown_sources_dir: String,
        rendered_outputs_dir: String,
    ) -> Result<Self, BlogGeneratorError> {
        let css_metadata = fs::metadata(&css_source_file);
        if !css_metadata.is_ok() {
            let e = css_metadata.unwrap_err();
            return Err(BlogGeneratorError::InvalidCSSPath(
                format!("{}", &css_source_file),
                e.to_string(),
            ));
        }

        let markdown_metadata = fs::metadata(&markdown_sources_dir);
        if !markdown_metadata.is_ok() {
            let e = markdown_metadata.unwrap_err();
            return Err(BlogGeneratorError::InvalidMarkDownPath(
                format!("{}", &markdown_sources_dir),
                e.to_string(),
            ));
        }

        let rendered_metadata = fs::metadata(&rendered_outputs_dir);
        if !rendered_metadata.is_ok() {
            let e = rendered_metadata.unwrap_err();
            return Err(BlogGeneratorError::InvalidRenderedOutputPath(
                format!("{}", &rendered_outputs_dir),
                e.to_string(),
            ));
        }

        Ok(BlogGenerator {
            base_url,
            css_source_file,
            markdown_sources_dir,
            rendered_outputs_dir,
        })
    }

    pub fn render(&self) -> Result<(), BlogGeneratorError> {
        let mut css_from_source = String::new();
        let css_f = File::open(&self.css_source_file);
        match css_f {
            Ok(mut css_f) => {
                let _ = css_f.read_to_string(&mut css_from_source).map_err(|e| {
                    BlogGeneratorError::CSSSourceError(
                        format!("{}", &self.css_source_file),
                        e.to_string(),
                    )
                });
                //println!("{:?}", &css_from_source);
            }
            Err(e) => {
                return Err(BlogGeneratorError::InvalidCSSPath(
                    format!("{}", &self.css_source_file),
                    e.to_string(),
                ))
            }
        }

        let mut markdown_files: Vec<MarkDownFile> = Vec::new();
        let md_glob_path = format!("{}/{}", &self.markdown_sources_dir, "*.md");

        // for each markdown file, append to a vec of MarkDownFile type
        // later sort by creation date
        for entry in glob(&md_glob_path).expect("failed to read any markdown source files") {
            match entry {
                Ok(path) => {
                    //let file_path = format!("{:?}", &path.display());
                    //println!("{:?}", &file_path);

                    let f = File::open(&path).map_err(|e| {
                        BlogGeneratorError::MarkDownFileError(
                            format!("{}", &path.display()),
                            e.to_string(),
                        )
                    });

                    let f_metadata = f.unwrap().metadata().map_err(|e| {
                        BlogGeneratorError::MarkDownMetadataError(
                            format!("{}", &path.display()),
                            e.to_string(),
                        )
                    });

                    let created_at = f_metadata.unwrap().created().unwrap();
                    let created_time: DateTime<Utc> = created_at.into();

                    let mdf = MarkDownFile {
                        file_name: PathBuf::from(&path.file_name().unwrap()),
                        file_path_buf: path.clone(),
                        created_time,
                        title_from_md: None,
                    };

                    markdown_files.push(mdf);
                }
                Err(e) => {
                    println!("{:?}", e);
                    continue;
                }
            }
        }

        // sort the vector of markdown files by created date
        let mut markdown_files_sorted = markdown_files.clone();
        markdown_files_sorted.sort_by(|a, b| a.created_time.cmp(&b.created_time));

        let mut tera = Tera::default();
        let _ = tera
            .add_raw_template("html", get_html_template())
            .map_err(|e| BlogGeneratorError::TemplateAddError("html".to_string(), e.to_string()));

        let mut pages: Vec<Page> = Vec::new();

        for mdf in markdown_files_sorted.iter_mut() {
            let mut md_content = String::new();
            let markdown_f = File::open(&*mdf.file_path_buf).map_err(|e| {
                BlogGeneratorError::InvalidMarkDownPath(
                    format!("{}", &mdf.file_path_buf.display()),
                    e.to_string(),
                )
            });

            if let Ok(mut markdown_f) = markdown_f {
                let _ = markdown_f.read_to_string(&mut md_content).map_err(|e| {
                    BlogGeneratorError::MarkDownFileError(
                        format!("{}", &mdf.file_path_buf.display()),
                        e.to_string(),
                    )
                });
            }

            let mut options = Options::empty();
            options.insert(Options::ENABLE_STRIKETHROUGH);
            options.insert(Options::ENABLE_TABLES);
            options.insert(Options::ENABLE_FOOTNOTES);
            options.insert(Options::ENABLE_TASKLISTS);
            options.insert(Options::ENABLE_SMART_PUNCTUATION);

            let parser = Parser::new_ext(&md_content, options);
            let mut body_content = String::new();
            html::push_html(&mut body_content, parser);

            // also try and scrape out the title from the markdown file
            let fragment = Html::parse_fragment(&body_content);
            if let Ok(selector) = Selector::parse("h1") {
                let h1 = fragment.select(&selector).next().unwrap();
                let title_text: Vec<&str> = h1.text().collect::<Vec<_>>();
                let title_text = format!("{:?}", title_text[0]);
                println!("Entry title: {:?}", &title_text);
                mdf.title_from_md = Some(title_text.clone());
            };

            // render the template
            let mut context = tera::Context::new();
            context.insert("body_content", &body_content);
            context.insert("css_from_source", &css_from_source);

            let rendered = tera.render("html", &context);
            if let Ok(rendered) = rendered {
                //println!("{:?}", &rendered);
                let title = &*mdf.title_from_md.clone().unwrap();
                let out_file_name = &*mdf.file_name.to_str().unwrap().replace(".md", ".html");
                let out_file_name: String = out_file_name.to_string();

                let out_path = format!("{}/{}", &self.rendered_outputs_dir, &out_file_name);

                let f = File::create(&out_path).map_err(|e| {
                    BlogGeneratorError::FileWriteError(out_path.to_string(), e.to_string())
                });

                let f_write = f.unwrap().write_all(&rendered.as_bytes());
                match f_write {
                    Ok(_f) => {
                        println!("wrote {:?}", &out_path);
                        let page = Page {
                            title: title.to_string().replace("\"", ""),
                            url: format!("{}{}", &self.base_url, out_file_name.clone()),
                        };
                        pages.push(page);
                    }
                    Err(e) => {
                        println!("error writing rendered file {:?}: {}", &out_path, e);
                        continue;
                    }
                }
            } else {
                return Err(BlogGeneratorError::TemplateUseError(
                    "html".to_string(),
                    "".to_string(),
                ));
            }
        }

        // generate an index page that contains links to all the pages, sorted by creation time
        let _ = tera
            .add_raw_template("index", get_index_page_template())
            .map_err(|e| BlogGeneratorError::TemplateAddError("index".to_string(), e.to_string()));

        let mut context = tera::Context::new();
        context.insert("pages", &pages);

        let rendered = tera.render("index", &context);
        if let Ok(rendered) = rendered {
            let out_file = format!("{}/index.html", &self.rendered_outputs_dir);

            let f = File::create(&out_file).map_err(|e| {
                BlogGeneratorError::FileWriteError(out_file.to_string(), e.to_string())
            });

            let _ = f.unwrap().write_all(&rendered.as_bytes()).map_err(|e| {
                BlogGeneratorError::FileWriteError(out_file.to_string(), e.to_string())
            });
        } else {
            return Err(BlogGeneratorError::TemplateUseError(
                "index".to_string(),
                "".to_string(),
            ));
        }

        Ok(())
    }
}
