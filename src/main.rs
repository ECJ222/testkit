#![feature(extend_one)]

mod base_cli;
mod base_request;
use base_cli::Commands;
use base_request::TestContext;
use clap::Parser;
use dotenv::dotenv;
use log::LevelFilter;
use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};
use walkdir::WalkDir;

mod app;
extern crate log;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let cli_instance = base_cli::Cli::parse();

    let mut builder = env_logger::Builder::from_default_env();
    builder
        .format_timestamp(None)
        .format_target(true)
        .filter_level(LevelFilter::from_str(&cli_instance.log_level).unwrap_or(LevelFilter::Info))
        .filter_module("jsonpath_lib", LevelFilter::Info)
        .init();

    match cli_instance.command {
        None | Some(Commands::App {}) => {
            app::app_init();
        }
        Some(Commands::Test { file }) => cli(file).await.unwrap(),
    }
}

async fn cli(file: PathBuf) -> Result<(), anyhow::Error> {
    if file.exists() {
        let content = fs::read_to_string(file.clone())?;
        let ctx = TestContext {
            file: file.to_str().unwrap().into(),
            file_source: content.clone(),
            ..Default::default()
        };
        base_request::run(ctx, content).await
    } else {
        let files = find_tk_yaml_files(Path::new("."));
        for file in files {
            let content = fs::read_to_string(file.clone())?;
            let ctx = TestContext {
                file: file.to_str().unwrap().into(),
                file_source: content.clone(),
                ..Default::default()
            };
            let _ = base_request::run(ctx, content).await;
        }
        Ok(())
    }
}

fn find_tk_yaml_files(dir: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Some(extension) = entry.path().extension() {
                if extension == "yaml"
                    && entry
                        .path()
                        .file_stem()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .contains(".tk")
                {
                    result.push(entry.path().to_path_buf());
                }
            }
        }
    }
    result
}
