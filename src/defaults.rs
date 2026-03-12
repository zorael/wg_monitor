//! Default values for various settings and configurations used throughout the program.
//!
//! This is just to gather them in one neat place.

use std::time;

/// Default Wireguard interface name.
pub const INTERFACE: &str = "wg0";

/// Default timeout duration for monitoring checks.
pub const TIMEOUT: time::Duration = time::Duration::from_secs(600);

/// Default check interval for monitoring the Wireguard interface.
pub const CHECK_INTERVAL: time::Duration = time::Duration::from_secs(60);

/// Default reminder interval for sending reminder notifications. Base value, will be grown.
pub const REMINDER_INTERVAL: time::Duration = time::Duration::from_secs(3600 * 6);

/// Base retry interval. Will be grown as retry attempts increase.
pub const RETRY_INTERVAL: time::Duration = time::Duration::from_secs(60); // 1m

/// Default configuration file name.
pub const CONFIG_FILENAME: &str = "config.toml";

/// Default peer list file name.
pub const PEER_LIST_FILENAME: &str = "peers.txt";

/// Default content for the peer list file.
pub const EMPTY_PEER_LIST_CONTENT: &str = "# <public key> <description>\n\
    # PeerKey/rc0fVvSsnw0xyzElf1vmtFbAe9w7cz+BXg7= Humanly-readable description of peer\n";

/// Default URL for testing Slack notifications with a dummy webhook URL.
pub const DUMMY_SLACK_URL: &str = "https://hooks.slack.com/services/DUMMY/HOOK/url";

/// Default URL for testing Batsign notifications with a dummy Batsign URL.
pub const DUMMY_BATSIGN_URL: &str = "https://batsign.me/at/example@mail.tld/asdf1234";

/// Default command for testing Command notifications with a dummy command.
pub const DUMMY_COMMAND: &str = "/usr/bin/echo";

pub mod program_metadata {
    //! Program metadata constants, such as the program name, version, authors, and source repository URL.

    use constcat::concat;

    /// The name of the program, as specified in Cargo.toml.
    pub const PROGRAM_NAME: &str = env!("CARGO_PKG_NAME");

    /// The program name as it appears in `argv[0]`.
    pub const PROGRAM_ARG0: &str = env!("CARGO_PKG_NAME");

    /// The authors of the program, as specified in Cargo.toml.
    pub const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

    /// A semver patch string to append to the version for pre-release versions.
    /// This should be empty for stable releases and can be set to something
    /// like "-beta.01" for pre-release versions.
    pub const SEMVER_PATCH: &str = "";

    /// The version of the program, as specified in Cargo.toml,
    /// with an optional semver patch for pre-release versions.
    pub const VERSION: &str = concat!("v", env!("CARGO_PKG_VERSION"), SEMVER_PATCH);

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
    pub const FAILED_TO_RESOLVE_CONFIG_DIR: u8 = 40;
    pub const CONFIG_DIR_DOES_NOT_EXIST: u8 = 41;
    pub const FAILED_TO_LOAD_RESOURCES: u8 = 42;
    pub const FAILED_TO_READ_CONFIG_FILE: u8 = 43;
    pub const CONFIG_FILE_DOES_NOT_EXIST: u8 = 44;
    pub const INSUFFICIENT_PRIVILEGES: u8 = 50;
}
