use anyhow::{bail, Context, Result};
use log::{debug, error};
use mdbook_preprocessor::book::Book;
use mdbook_preprocessor::{Preprocessor, PreprocessorContext};
use std::env;
use std::io::{self, Read};

use mdbook_drawio::DrawioPreprocessor;

fn main() -> Result<()> {
    // Log to stderr, which mdBook passes through. Set `MDBOOK_DRAWIO_LOG=debug`
    // for more detail.
    env_logger::Builder::from_env(env_logger::Env::new().filter_or("MDBOOK_DRAWIO_LOG", "info"))
        .format_timestamp(None)
        .format_target(false)
        .init();

    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        match args[1].as_str() {
            "supports" => {
                // mdBook only looks at the exit code, and prints anything we
                // write to stdout.
                debug!("mdbook-drawio supports command called");
                return Ok(());
            }
            "preprocess" => {
                debug!("mdbook-drawio preprocess command called");
            }
            _ => {}
        }
    }

    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .context("reading stdin")?;
    if input.trim().is_empty() {
        bail!("empty stdin for preprocess");
    }

    let (ctx, book): (PreprocessorContext, Book) =
        serde_json::from_str(&input).context("parsing ctx/book JSON")?;

    debug!("Starting mdbook-drawio preprocessing");
    debug!("Book context: {:?}", ctx);

    let pre = DrawioPreprocessor;
    let out = pre.run(&ctx, book).map_err(|e| {
        error!("Preprocessing failed: {}", e);
        anyhow::anyhow!("{}", e)
    })?;

    debug!("Preprocessing completed successfully");
    println!("{}", serde_json::to_string(&out)?);
    Ok(())
}
