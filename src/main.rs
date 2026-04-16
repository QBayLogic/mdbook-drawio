use anyhow::{bail, Context, Result};
use log::{debug, error};
use mdbook_preprocessor::book::Book;
use mdbook_preprocessor::{Preprocessor, PreprocessorContext};
use std::env;
use std::io::{self, Read};

use mdbook_drawio::{get_result_dir_abs, DrawioPreprocessor};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        match args[1].as_str() {
            "supports" => {
                debug!("mdbook-drawio supports command called");
                println!("true");
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

    let result_dir = get_result_dir_abs(&ctx);
    // Ensure the result directory exists
    std::fs::create_dir_all(&result_dir)?;

    let log_path = result_dir.join("mdbook-drawio.log");

    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .target(env_logger::Target::Pipe(Box::new(std::fs::File::create(
            log_path,
        )?)))
        .init();

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
