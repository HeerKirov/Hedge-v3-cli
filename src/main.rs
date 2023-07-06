mod cli;
mod command;
mod module;

use clap::Parser;
use cli::{Cli, Import, Channel, Server};
use command::apply::ApplyInputType;
use module::local_data::LocalDataManager;
use module::channel::ChannelManager;
use module::server::ServerManager;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let config = module::config::load_config();
    let local_data_manager = LocalDataManager::new(&config);
    let channel_manager = ChannelManager::new(&config, &local_data_manager);
    let mut server_manager = ServerManager::new(&config, &channel_manager);
    let mut context = command::Context {
        config,
        local_data_manager: &local_data_manager,
        channel_manager: &channel_manager,
        server_manager: &mut server_manager
    };
    match cli {
        Cli::App => command::app::start_app(&context),
        Cli::Channel(channel) => match channel {
            Channel::Info => command::channel::info(&context),
            Channel::Use { channel_name } => command::channel::use_channel(&context, channel_name)
        }
        Cli::Server(server) => match server {
            Server::Status => command::server::status(&mut context).await,
            Server::Start => command::server::start(&mut context).await,
            Server::Stop => command::server::stop(&mut context).await
        }
        Cli::Apply(apply) => {
            if let Some(f) = apply.directory { 
                command::apply::apply(&context, ApplyInputType::Directory(f), apply.quiet)
            }else if let Some(f) = apply.file {
                command::apply::apply(&context, ApplyInputType::File(f), apply.quiet)
            }else if apply.input{
                command::apply::apply(&context, ApplyInputType::Input, apply.quiet)
            }else{
                eprintln!("Options --directory, --file and --input should have least one.")
            }
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