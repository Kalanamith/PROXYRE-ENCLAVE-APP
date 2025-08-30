use clap::ArgMatches;

#[derive(Debug, Clone, PartialEq)]
pub struct ServerArgs {
    pub port: u32,
}

impl ServerArgs {
    pub fn new_with(args: &ArgMatches) -> Result<Self, String> {
        Ok(ServerArgs {
            port: parse_port(args)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientArgs {
    pub cid: u32,
    pub port: u32,
}

impl ClientArgs {
    pub fn new_with(args: &ArgMatches) -> Result<Self, String> {
        Ok(ClientArgs {
            cid: parse_cid_client(args)?,
            port: parse_port(args)?,
        })
    }
}

fn parse_cid_client(args: &ArgMatches) -> Result<u32, String> {
    let cid_str = args.get_one::<String>("cid").ok_or("Could not find cid argument")?;
    cid_str.parse()
        .map_err(|_err| "cid is not a number".to_string())
}

fn parse_port(args: &ArgMatches) -> Result<u32, String> {
    let port_str = args
        .get_one::<String>("port")
        .ok_or("Could not find port argument")?;
    port_str.parse()
        .map_err(|_err| "port is not a number".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Command;

    // Test ServerArgs struct
    #[test]
    fn test_server_args_creation() {
        let args = ServerArgs { port: 8080 };
        assert_eq!(args.port, 8080);
    }

    #[test]
    fn test_server_args_debug() {
        let args = ServerArgs { port: 5005 };
        let debug_str = format!("{:?}", args);
        assert!(debug_str.contains("5005"));
    }

    // Test ClientArgs struct
    #[test]
    fn test_client_args_creation() {
        let args = ClientArgs {
            cid: 123,
            port: 8080,
        };
        assert_eq!(args.cid, 123);
        assert_eq!(args.port, 8080);
    }

    #[test]
    fn test_client_args_debug() {
        let args = ClientArgs {
            cid: 456,
            port: 3000,
        };
        let debug_str = format!("{:?}", args);
        assert!(debug_str.contains("456"));
        assert!(debug_str.contains("3000"));
    }

    // Test partial argument parsing (individual functions)
    #[test]
    fn test_parse_cid_client_edge_cases() {
        // Test with valid numeric string
        let app = Command::new("test")
            .arg(clap::Arg::new("cid").long("cid").required(true));

        let matches = app.clone().try_get_matches_from(vec!["test", "--cid", "0"]).unwrap();
        assert_eq!(parse_cid_client(&matches).unwrap(), 0);

        let matches = app.try_get_matches_from(vec!["test", "--cid", "999999"]).unwrap();
        assert_eq!(parse_cid_client(&matches).unwrap(), 999999);
    }

    #[test]
    fn test_parse_port_edge_cases() {
        // Test with valid numeric string
        let app = Command::new("test")
            .arg(clap::Arg::new("port").long("port").required(true));

        let matches = app.clone().try_get_matches_from(vec!["test", "--port", "1"]).unwrap();
        assert_eq!(parse_port(&matches).unwrap(), 1);

        let matches = app.try_get_matches_from(vec!["test", "--port", "65535"]).unwrap();
        assert_eq!(parse_port(&matches).unwrap(), 65535);
    }

    // Test error messages
    #[test]
    fn test_parse_cid_error_message() {
        let app = Command::new("test")
            .arg(clap::Arg::new("cid").long("cid").required(true));

        let matches = app.try_get_matches_from(vec!["test", "--cid", "not_a_number"]).unwrap();
        let error = parse_cid_client(&matches).unwrap_err();
        assert!(error.contains("cid is not a number"));
    }

    #[test]
    fn test_parse_port_error_message() {
        let app = Command::new("test")
            .arg(clap::Arg::new("port").long("port").required(true));

        let matches = app.try_get_matches_from(vec!["test", "--port", "not_a_number"]).unwrap();
        let error = parse_port(&matches).unwrap_err();
        assert!(error.contains("port is not a number"));
    }

    // Test missing arguments
    #[test]
    fn test_parse_cid_missing_argument() {
        let app = Command::new("test")
            .arg(clap::Arg::new("cid").long("cid").required(false)); // Make it optional to get matches

        let matches = app.try_get_matches_from(vec!["test"]).unwrap();
        let error = parse_cid_client(&matches).unwrap_err();
        assert!(error.contains("Could not find cid argument"));
    }

    #[test]
    fn test_parse_port_missing_argument() {
        let app = Command::new("test")
            .arg(clap::Arg::new("port").long("port").required(false)); // Make it optional to get matches

        let matches = app.try_get_matches_from(vec!["test"]).unwrap();
        let error = parse_port(&matches).unwrap_err();
        assert!(error.contains("Could not find port argument"));
    }

    // Test struct implementations
    #[test]
    fn test_server_args_clone() {
        let args1 = ServerArgs { port: 8080 };
        let args2 = args1.clone();
        assert_eq!(args1.port, args2.port);
    }

    #[test]
    fn test_client_args_clone() {
        let args1 = ClientArgs {
            cid: 123,
            port: 8080,
        };
        let args2 = args1.clone();
        assert_eq!(args1.cid, args2.cid);
        assert_eq!(args1.port, args2.port);
    }

    // Test equality
    #[test]
    fn test_server_args_equality() {
        let args1 = ServerArgs { port: 8080 };
        let args2 = ServerArgs { port: 8080 };
        let args3 = ServerArgs { port: 9090 };

        assert_eq!(args1, args2);
        assert_ne!(args1, args3);
    }

    #[test]
    fn test_client_args_equality() {
        let args1 = ClientArgs {
            cid: 123,
            port: 8080,
        };
        let args2 = ClientArgs {
            cid: 123,
            port: 8080,
        };
        let args3 = ClientArgs {
            cid: 456,
            port: 8080,
        };

        assert_eq!(args1, args2);
        assert_ne!(args1, args3);
    }
}
