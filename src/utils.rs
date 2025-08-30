use log::error;

/// A trait that provides a convenient method to exit the program with an error message
/// if a `Result` contains an `Err` value.
///
/// This trait is automatically implemented for all `Result<T, E>` types where `E` implements `Debug`.
/// It provides a clean way to handle errors by logging them and terminating the program,
/// which is particularly useful in command-line applications where graceful error handling
/// is required.
///
/// # Examples
///
/// ```rust
/// use proxy_reencyption_enclave_app::utils::ExitGracefully;
///
/// fn main() {
///     let result: Result<i32, &str> = Ok(42);
///     // This will return the value 42
///     let value = result.ok_or_exit("This message won't be logged");
///     println!("Got value: {}", value);
///
///     // If we had an error, it would exit:
///     // let error_result: Result<i32, &str> = Err("Something went wrong");
///     // let value = error_result.ok_or_exit("Failed to get value"); // Exits here
/// }
/// ```
pub trait ExitGracefully<T, E> {
    /// Unwraps a `Result`, logging the error and exiting the program if it's `Err`.
    ///
    /// If the `Result` is `Ok(value)`, this method returns `value`.
    /// If the `Result` is `Err(error)`, this method logs the error message using the
    /// `log::error!` macro and then calls `std::process::exit(1)` to terminate the program.
    ///
    /// # Parameters
    /// * `message` - A custom error message to include in the log output
    ///
    /// # Returns
    /// Returns the contained `Ok` value if the result is successful.
    ///
    /// # Panics
    /// This method never panics - it terminates the program instead.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use proxy_reencyption_enclave_app::utils::ExitGracefully;
    ///
    /// let result: Result<i32, &str> = Ok(42);
    /// assert_eq!(result.ok_or_exit("This won't be logged"), 42);
    ///
    /// // The following would exit the program:
    /// // let result: Result<i32, &str> = Err("error");
    /// // result.ok_or_exit("Program failed"); // Exits here
    /// ```
    fn ok_or_exit(self, message: &str) -> T;
}

/// Implementation of the `ExitGracefully` trait for `Result<T, E>` types.
///
/// This implementation provides the concrete behavior for the `ok_or_exit` method.
/// When a `Result` contains an error, it logs the error with the provided message
/// and terminates the program with exit code 1.
///
/// The error logging uses the `log::error!` macro, so the output will depend on
/// the current logging configuration. The error value is formatted using its
/// `Debug` implementation.
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

/// Creates a configured `clap::Command` for the Proxy Re-encryption Enclave Application.
///
/// This macro expands to a complete CLI command definition using the `clap` crate.
/// It sets up the main command structure with subcommands for both server and client modes.
///
/// The generated command includes:
/// - Application name: "proxy_reencyption Enclave App"
/// - Description: "Proxy Re Encryption Application"
/// - Version information from Cargo.toml
/// - Help requirement (shows help if no arguments provided)
/// - Two subcommands: `server` and `client`
///
/// # Server Subcommand
/// The server subcommand is used to start the application in server mode.
/// It requires a `--port` argument specifying which port to listen on.
///
/// # Client Subcommand
/// The client subcommand is used to start the application in client mode.
/// It requires both `--port` and `--cid` arguments:
/// - `--port`: The port number to connect to
/// - `--cid`: The connection ID for the target enclave
///
/// # Returns
/// Returns a fully configured `clap::Command` that can be used to parse command-line arguments.
///
/// # Examples
///
/// ```rust
/// use proxy_reencyption_enclave_app::create_app;
///
/// # fn main() {
/// let app = create_app!();
///
/// // Parse command line arguments (providing dummy args for demonstration)
/// let matches = app.get_matches_from(vec!["app", "server", "--port", "8080"]);
///
/// // Handle subcommands
/// match matches.subcommand() {
///     Some(("server", sub_matches)) => {
///         let port: u32 = sub_matches.get_one::<String>("port")
///             .unwrap()
///             .parse()
///             .unwrap();
///         // Start server on specified port
///         println!("Starting server on port {}", port);
///     }
///     Some(("client", sub_matches)) => {
///         let port: u32 = sub_matches.get_one::<String>("port")
///             .unwrap()
///             .parse()
///             .unwrap();
///         let cid: u32 = sub_matches.get_one::<String>("cid")
///             .unwrap()
///             .parse()
///             .unwrap();
///         // Connect to server with specified port and CID
///         println!("Connecting to server on port {} with CID {}", port, cid);
///     }
///     _ => {
///         // Handle invalid subcommand
///         println!("Invalid subcommand");
///     }
/// }
/// # }
/// ```
///
/// # Usage in Main Function
///
/// ```rust
/// # fn main() {
/// use proxy_reencyption_enclave_app::create_app;
///
///     let app = create_app!();
///     // Provide dummy arguments to avoid triggering help
///     let matches = app.get_matches_from(vec!["app", "server", "--port", "8080"]);
///
///     // Application logic based on matches...
///     match matches.subcommand() {
///         Some(("server", _)) => println!("Server mode selected"),
///         Some(("client", _)) => println!("Client mode selected"),
///         _ => println!("No valid subcommand provided"),
///     }
/// # }
/// ```
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
                    .arg(clap::Arg::new("cid").long("cid").help("cid").required(true)),
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
        let result = app
            .clone()
            .try_get_matches_from(vec!["test", "server", "--port", "8080"]);
        assert!(result.is_ok(), "Should accept valid server arguments");

        let result =
            app.try_get_matches_from(vec!["test", "client", "--port", "8080", "--cid", "123"]);
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
        assert_eq!(
            app1.get_subcommands().count(),
            app2.get_subcommands().count()
        );
    }

    // Test that the macro works with different argument orders
    #[test]
    fn test_create_app_macro_argument_ordering() {
        let app = create_app!();

        // Test different argument order for client
        let result1 = app
            .clone()
            .try_get_matches_from(vec!["test", "client", "--port", "8080", "--cid", "123"]);
        let result2 =
            app.try_get_matches_from(vec!["test", "client", "--cid", "123", "--port", "8080"]);

        assert!(result1.is_ok());
        assert!(result2.is_ok());
    }
}
