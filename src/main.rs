use anyhow::Result;
use clap::Parser;

use md_blog_gen::blog_gen::blog_generator::BlogGenerator;

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    #[arg(short, long, help = "path to the CSS source file")]
    css_source: String,

    #[arg(short, long, help = "path to the dir containing the markdown files")]
    md_sources: String,

    #[arg(
        short,
        long,
        help = "path to the dir into which the rendered files will be written"
    )]
    rendered_outputs: String,
}

fn main() -> Result<()> {
    // some example paths
    //let css_source = "/home/pimeson/Development/RustDev/md-blog-gen/md-blog-gen/css_sources/retro.css".to_string();
    //let markdown_sources = "/home/pimeson/Development/RustDev/md-blog-gen/md-blog-gen/md_sources".to_string();
    //let rendered_outputs = "/home/pimeson/Development/RustDev/md-blog-gen/md-blog-gen/rendered_html".to_string();

    let args = Args::parse();

    let br = BlogGenerator::new(
        "./".to_string(),
        args.css_source,
        args.md_sources,
        args.rendered_outputs,
    )
    .map_err(|e| eprintln!("{}", e));

    if let Ok(r) = br {
        r.render()?;
    };

    Ok(())
}
