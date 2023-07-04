mod config;

use std::path::PathBuf;
use clap::{Args, Parser, Subcommand, ValueEnum};
use chrono::NaiveDate;

#[derive(Parser)]
#[command(bin_name = "hedge", name = "hedge", version, about = "Hedge Command Line Application")]
enum Cli {
    #[command(about = "Start hedge application")]
    App,
    #[command(about = "Apply anywhere files for data updates")]
    Apply(Apply),
    #[command(subcommand, about = "Cli Channel control")]
    Channel(Channel),
    #[command(subcommand, about = "Background service process control")]
    Server(Server),
    #[command(subcommand, about = "File import management")]
    Import(Import),
    #[command(subcommand, about = "Source data management")]
    SourceData(SourceData)
}

#[derive(Args)]
struct Apply {
    #[arg(short, long, help = "read from files in directory")]
    directory: Option<PathBuf>,
    #[arg(short, long, help = "read from file")]
    file: Option<PathBuf>,
    #[arg(short, long, help = "read from stdin")]
    input: Option<PathBuf>,
    #[arg(short, long, help = "don't print any output")]
    quiet: bool,
}

#[derive(Subcommand)]
enum Channel {
    #[command(about = "Show using and all channels")]
    Info,
    #[command(about = "Change using channel")]
    Use {
        #[arg(help = "target channel name")]
        channel_name: String
    }
}

#[derive(Subcommand)]
enum Server {
    #[command(about = "Show service status")]
    Status,
    #[command(about = "Start service and keep it running")]
    Start,
    #[command(about = "Stop service")]
    Stop
}

#[derive(Subcommand)]
enum Import {
    #[command(about = "Import new file")]
    Add {
        #[arg(help = "any local files")]
        files: Vec<PathBuf>,
        #[arg(short, long, help = "remove origin file")]
        remove: bool
    },
    #[command(about = "List all imported files")]
    List,
    #[command(about = "Batch update imported files")]
    Batch {
        #[arg(short, long, help = "set partition time")]
        partition_time: Option<NaiveDate>,
        #[arg(short, long, help = "set create time by some category")]
        create_time: Option<OrderTimeType>,
        #[arg(short, long, help = "set order time by some category")]
        order_time: Option<OrderTimeType>,
        #[arg(short, long, help = "analyse source date")]
        analyse_source: bool
    },
    #[command(about = "Save all imported files")]
    Save
}

#[derive(Subcommand)]
enum SourceData {
    #[command(about = "Query source data by HQL")]
    Query {
        #[arg(help = "hedge query language")]
        hql: String,
        #[arg(long, help = "query limit", default_value_t = 100)]
        limit: u32,
        #[arg(long, help = "query offset", default_value_t = 0)]
        offset: u32
    },
    #[command(about = "Download metadata for NOT_EDITED source data")]
    Download,
    #[command(about = "Connect database to read metadata for NOT_EDITED source data")]
    Connect
}

#[derive(Clone, ValueEnum)]
enum OrderTimeType {
    CreateTime,
    UpdateTime,
    ImportTime
}

fn main() {
    let cli = Cli::parse();
    let _config = config::load_config();
    match cli {
        Cli::App => {

        }
        Cli::Apply(apply) => {
            if let Some(f) = apply.file {
                println!("file: {}", f.to_str().unwrap())
            }
        }
        Cli::Channel(_) => {

        }
        Cli::Server(_) => {

        }
        Cli::Import(import) => match import {
            Import::Add { .. } => {

            }
            Import::List => {

            }
            Import::Batch { .. } => {

            }
            Import::Save => {

            }
        }
        Cli::SourceData(_) => {

        }
    }
}