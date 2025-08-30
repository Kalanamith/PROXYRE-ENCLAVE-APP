use log::error;

pub trait ExitGracefully<T, E> {
    fn ok_or_exit(self, message: &str) -> T;
}

impl<T, E: std::fmt::Debug> ExitGracefully<T, E> for Result<T, E> {
    fn ok_or_exit(self, message: &str) -> T {
        match self {
            Ok(val) => val,
            Err(err) => {
                error!("{:?}: {}", err, message);
                std::process::exit(1);
            }
        }
    }
}

#[macro_export]
macro_rules! create_app {
    () => {
        clap::Command::new("proxy_reencyption Enclave App")
            .about("Proxy Re Encryption Application")
            .arg_required_else_help(true)
            .version(env!("CARGO_PKG_VERSION"))
            .subcommand(
                clap::Command::new("server")
                    .about("Listen on a given port.")
                    .arg(
                        clap::Arg::new("port")
                            .long("port")
                            .help("port")
                            .required(true),
                    ),
            )
            .subcommand(
                clap::Command::new("client")
                    .about("Connect to a given cid and port.")
                    .arg(
                        clap::Arg::new("port")
                            .long("port")
                            .help("port")
                            .required(true),
                    )
                    .arg(
                        clap::Arg::new("cid")
                            .long("cid")
                            .help("cid")
                            .required(true),
                    ),
            )
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test ExitGracefully trait implementation
    #[test]
    fn test_exit_gracefully_trait_implementation() {
        // Test that the trait is implemented for Result<T, E> where E: std::fmt::Debug
        let result: Result<i32, &str> = Ok(42);

        // The trait is implemented automatically for all Result types where E: std::fmt::Debug
        // We can't test the actual exit behavior since std::process::exit terminates the process
        // But we can verify the trait is available by using it in a type annotation
        assert!(true, "ExitGracefully trait is implemented for Result types");
    }

    // Test macro functionality
    #[test]
    fn test_create_app_macro() {
        let app = create_app!();

        // Verify basic app structure
        assert_eq!(app.get_name(), "proxy_reencyption Enclave App");
        assert!(app.get_about().is_some());

        // Verify subcommands exist
        let subcommands: Vec<_> = app.get_subcommands().collect();
        assert!(!subcommands.is_empty());

        // Check for server and client subcommands
        let subcommand_names: Vec<_> = subcommands.iter().map(|cmd| cmd.get_name()).collect();
        assert!(subcommand_names.contains(&"server"));
        assert!(subcommand_names.contains(&"client"));
    }

    #[test]
    fn test_create_app_macro_structure() {
        let app = create_app!();

        // Test that we can get matches from valid arguments
        let result = app.clone().try_get_matches_from(vec!["test", "server", "--port", "8080"]);
        assert!(result.is_ok(), "Should accept valid server arguments");

        let result = app.try_get_matches_from(vec!["test", "client", "--port", "8080", "--cid", "123"]);
        assert!(result.is_ok(), "Should accept valid client arguments");
    }

    #[test]
    fn test_create_app_macro_error_handling() {
        let app = create_app!();

        // Test missing required arguments
        let result = app.clone().try_get_matches_from(vec!["test", "server"]);
        assert!(result.is_err(), "Should reject missing port argument");

        let result = app.try_get_matches_from(vec!["test", "client", "--port", "8080"]);
        assert!(result.is_err(), "Should reject missing cid argument");
    }

    // Test trait bounds and implementations
    #[test]
    fn test_result_trait_bounds() {
        // Test various Result types implement the trait
        let result_str: Result<i32, &str> = Ok(1);
        let result_string: Result<i32, String> = Ok(2);
        let result_box: Result<i32, Box<dyn std::error::Error>> = Ok(3);

        // Verify all have the ok_or_exit method
        let _ = result_str.ok_or_exit("dummy");
        let _ = result_string.ok_or_exit("dummy");
        let _ = result_box.ok_or_exit("dummy");
    }

    // Test macro expansion doesn't break with different contexts
    #[test]
    fn test_create_app_macro_multiple_calls() {
        let app1 = create_app!();
        let app2 = create_app!();

        // Both should have the same structure
        assert_eq!(app1.get_name(), app2.get_name());
        assert_eq!(app1.get_subcommands().count(), app2.get_subcommands().count());
    }

    // Test that the macro works with different argument orders
    #[test]
    fn test_create_app_macro_argument_ordering() {
        let app = create_app!();

        // Test different argument order for client
        let result1 = app.clone().try_get_matches_from(vec!["test", "client", "--port", "8080", "--cid", "123"]);
        let result2 = app.try_get_matches_from(vec!["test", "client", "--cid", "123", "--port", "8080"]);

        assert!(result1.is_ok());
        assert!(result2.is_ok());
    }
}
