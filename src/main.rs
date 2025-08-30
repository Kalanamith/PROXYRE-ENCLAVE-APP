use clap::{Arg, ArgMatches, Command};

use proxy_reencyption_enclave_app::command_parser::{ClientArgs, ServerArgs};
use proxy_reencyption_enclave_app::create_app;
use proxy_reencyption_enclave_app::utils::ExitGracefully;
use proxy_reencyption_enclave_app::{client, server};

#[tokio::main]
async fn main() {
    let app = create_app!();
    let args = app.get_matches();

    match args.subcommand() {
        Some(("server", sub_matches)) => {
            let server_args =
                ServerArgs::new_with(sub_matches).ok_or_exit("Invalid server arguments");
            server(server_args).ok_or_exit("Server failed to start");
        }
        Some(("client", sub_matches)) => {
            let client_args =
                ClientArgs::new_with(sub_matches).ok_or_exit("Invalid client arguments");
            client(client_args)
                .await
                .ok_or_exit("Client failed to start");
        }
        _ => {}
    }
}
