use clap::{App, AppSettings, Arg, SubCommand};

use proxy_reencyption_enclave_app::command_parser::{ClientArgs, ServerArgs};
use proxy_reencyption_enclave_app::create_app;
use proxy_reencyption_enclave_app::utils::ExitGracefully;
use proxy_reencyption_enclave_app::{client, server};

fn main() {
    let app = create_app!();
    let args = app.get_matches();

    match args.subcommand() {
        ("server", Some(args)) => {
            let server_args = ServerArgs::new_with(args).ok_or_exit(args.usage());
            server(server_args).ok_or_exit(args.usage());
        }
        ("client", Some(args)) => {
            let client_args = ClientArgs::new_with(args).ok_or_exit(args.usage());
            client(client_args).ok_or_exit(args.usage());
        }
        (&_, _) => {}
    }
}
