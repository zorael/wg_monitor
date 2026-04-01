//! Default values.
//!
//! This is just to gather them in one place, and to avoid hardcoding these
//! values in multiple places throughout the program.

/// Default WireGuard interface name.
pub const INTERFACE: &str = "wg0";

/// Default path to the `wg` executable.
pub const WG_PATH: &str = "/usr/bin/wg";

/// Default configuration file name.
pub const CONFIG_FILENAME: &str = "config.toml";

/// Default peer list file name.
pub const PEER_LIST_FILENAME: &str = "peers.txt";

pub mod timing {
    //! Default timing values for various aspects of the program.

    use std::time;

    /// Default timeout duration for monitoring checks.
    pub const TIMEOUT: time::Duration = time::Duration::from_secs(600);

    /// Default check interval for monitoring the WireGuard interface.
    pub const CHECK_INTERVAL: time::Duration = time::Duration::from_secs(60);

    /// Default reminder interval for sending reminder notifications. Base value, will be grown.
    pub const REMINDER_INTERVAL: time::Duration = time::Duration::from_secs(3600 * 6);

    /// Base retry interval. Will be grown as retry attempts increase.
    pub const RETRY_INTERVAL: time::Duration = time::Duration::from_secs(10);

    /// Default delay between notification attempts for multiple notifiers of the
    /// same backend type, to avoid overwhelming the backend with simultaneous notifications.
    pub const NOTIFIER_RATE_LIMIT_DELAY: time::Duration = time::Duration::from_millis(300);
}

pub mod placeholder_values {
    //! Placeholder values for testing and default content.

    /// Default content for the peer list file.
    pub const EMPTY_PEER_LIST_CONTENT: &str = "# <public key> <description>\n\
        # PeerKey/rc0fVvSsnw0xyzElf1vmtFbAe9w7cz+BXg7= Humanly-readable description of peer\n";

    /// Default URL for testing Slack notifications with a dummy webhook URL.
    pub const DUMMY_SLACK_URL: &str = "https://hooks.slack.com/services/DUMMY/HOOK/url";

    /// Default URL for testing Batsign notifications with a dummy Batsign URL.
    pub const DUMMY_BATSIGN_URL: &str = "https://batsign.me/at/example@mail.tld/asdf1234";

    /// Default command for testing Command notifications with a dummy command.
    pub const DUMMY_COMMAND: &str = "/usr/bin/echo";
}

pub mod program_metadata {
    //! Program metadata constants, such as the program name, version, authors, and source repository URL.

    /// The name of the program, as specified in Cargo.toml.
    pub const PROGRAM_NAME: &str = env!("CARGO_PKG_NAME");

    /// The program name as it appears in `argv[0]`.
    pub const PROGRAM_ARG0: &str = env!("CARGO_PKG_NAME");

    /// The authors of the program, as specified in Cargo.toml.
    pub const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

    /// The version of the program, as specified in Cargo.toml.
    pub const VERSION: &str = env!("CARGO_PKG_VERSION");

    /// The source repository URL for the program, as specified in Cargo.toml.
    pub const SOURCE_REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
}

#[allow(dead_code)]
pub mod exit_codes {
    //! Exit codes for the program, used to indicate different types of errors or exit conditions.

    //pub const WRONG_PLATFORM: u8 = 100;
    pub const FAILED_TO_CREATE_CONFIG_DIR: u8 = 10;
    pub const FAILED_TO_WRITE_CONFIG_FILE: u8 = 11;
    pub const FAILED_TO_WRITE_PEER_LIST_FILE: u8 = 12;
    pub const CONFIGURATION_ERROR: u8 = 20;
    pub const NO_NOTIFIERS_CONFIGURED: u8 = 21;
    pub const ERROR_READING_PEERS_FILE: u8 = 30;
    pub const FAILED_TO_EXECUTE_HANDSHAKES_COMMAND: u8 = 31;
    pub const FAILED_TO_PARSE_HANDSHAKES_OUTPUT: u8 = 32;
    pub const EMPTY_PEER_LIST: u8 = 33;
    pub const WG_EXECUTABLE_NOT_FOUND: u8 = 34;
    pub const FAILED_TO_PARSE_PEER_LIST_FILE: u8 = 35;
    pub const FAILED_TO_RESOLVE_CONFIG_DIR: u8 = 40;
    pub const CONFIG_DIR_DOES_NOT_EXIST: u8 = 41;
    pub const FAILED_TO_LOAD_RESOURCES: u8 = 42;
    pub const FAILED_TO_READ_CONFIG_FILE: u8 = 43;
    pub const CONFIG_FILE_DOES_NOT_EXIST: u8 = 44;
    pub const INSUFFICIENT_PRIVILEGES: u8 = 50;
    pub const INVALID_NOTIFIER_CONFIGURATION: u8 = 60;
}
