# wg_monitor

Monitors other peers in a [**WireGuard**](https://www.wireguard.com) VPN and sends a notification if contact with a peer is lost.

The main purpose of this is to monitor Internet-connected locations for power outages, using WireGuard handshakes as a way for sites to phone home. Each site needs an always-on, always-online computer to act as a WireGuard peer, for which something like a [**Raspberry Pi Zero 2W**](https://www.raspberrypi.com/products/raspberry-pi-zero-2-w) is cheap and more than sufficient. (May require [cross-compilation](#cross-compilation).)

In a hub-and-spoke WireGuard configuration, this program should be run on the hub server, with an additional instance on at least one other *geographically disconnected* peer to monitor the hub. In other configurations, it can be run on any peer with visibility of other peers, but a secondary instance monitoring the first is recommended in any setup. If the hub loses power, it cannot report itself as being lost.

Peers must have a `PersistentKeepalive` setting in their WireGuard configuration with a value *comfortably lower* than the peer timeout of this program. This timeout is **10 minutes** by default.

Notifications can be sent as [**Slack**](https://docs.slack.dev/messaging/sending-messages-using-incoming-webhooks) messages, as short emails via [**Batsign**](https://batsign.me), and/or by invocation of an [**external command**](#external-command) (like `notify-send`, `wall` or `sendmail`).

## tl;dr

```text
Usage: wg_monitor [OPTIONS]

Options:
  -c, --config-dir <path>   Specify an alternative configuration directory
      --resume              Word the first notification as if the program was not just started
      --skip-first          Skip the first run and thus the first notification
      --disable-timestamps  Disable timestamps in terminal output
      --sleep <duration>    Sleep for a specified duration before starting the monitoring loop
      --show                Output configuration to screen and exit
  -v, --verbose             Print some additional information
  -d, --debug               Print much more additional information
      --dry-run             Perform a dry run, echoing what would be done
      --save                Write configuration to disk
  -V, --version             Display version information and exit
```

Pre-compiled binaries for `x86_64` and `aarch64` architectures are available under [**Releases**](https://github.com/zorael/wg_monitor/releases).

Create a [configuration file](#configtoml) and a [peer list](#peerstxt) by passing `--save`.

```sh
cargo run -- --save
```

## toc

* [compilation](#compilation)
  * [cross-compilation](#cross-compilation)
  * [`-j1`](#-j1)
* [configuration](#configuration)
  * [`config.toml`](#configtoml)
  * [`peers.txt`](#peerstxt)
* [backends](#backends)
  * [slack](#slack)
    * [formatting messages](#formatting-messages)
  * [batsign](#batsign)
    * [formatting mails](#formatting-mails)
  * [external command](#external-command)
    * [arguments](#arguments)
    * [example scripts](#example-scripts)
      * [notify-send-to-all-gui.sh](#notify-send-to-all-guish)
      * [notify-send-to-one.sh](#notify-send-to-onesh)
* [systemd](#systemd)
  * [starts too early](#starts-too-early)
* [ai](#ai)
* [todo](#todo)
* [license](#license)

## compilation

This project uses [**Cargo**](https://doc.rust-lang.org/cargo) for compilation and dependency management. Get it from your repositories, install it via [**Homebrew `rustup`**](https://formulae.brew.sh/formula/rustup), or download it with the official [`rustup`](https://rustup.rs) installation script.

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

You may have to add `$HOME/.cargo/bin` to your `$PATH`.

Use `cargo build` to build the project. This stores the resulting binary as `target/<profile>/wg_monitor`, where `<profile>` is one of `debug` or `release`, depending on what profile is being built. `debug` is the default; you can make it build in `release` mode with `--release`.

```sh
cargo build
cargo build --release
```

To compile the program and run it immediately, use `cargo run`. If you also want to pass command-line flags to the program, separate them from `cargo run` with [double dashes](https://www.gnu.org/software/bash/manual/html_node/Shell-Builtin-Commands.html) `--`.

```sh
cargo run -- --help
cargo run -- --save
```

You can find the binaries you compile with Cargo in the `target/<profile>/` subdirectory of the project, where `<profile>` is either `debug` or `release`, depending on what profile you built with.

See the [**systemd**](#systemd) section for instructions on how to set it up as a system daemon that is automatically started on boot.

### cross-compilation

A device like the Pi Zero 2W can *run* the program but does not have enough memory to compile it, at least not with default flags. You can probably still build it on such a Pi by adding swap and exercising a lot of patience, but the convenient way is to just cross-compile it on another computer and transferring the resulting binary.

Regrettably, manually setting up cross-compilation can be non-trivial. As such, use of one of [`cargo-cross`](https://github.com/cross-rs/cross) or [`cargo-zigbuild`](https://github.com/rust-cross/cargo-zigbuild) is recommended (but not required). For the latter you need to install a [**Zig**](https://ziglang.org) compiler. Refer to your repositories, alternatively install it via Homebrew (`brew install zig`).

Note that your `$CFLAGS` environment variable must not contain `-march=native` for all dependencies to successfully build.

```sh
cargo install cargo-cross
CFLAGS="-O2 -pipe" cargo cross build --target=aarch64-unknown-linux-gnu
```

```sh
cargo install cargo-zigbuild
CFLAGS="-O2 -pipe" cargo zigbuild --target=aarch64-unknown-linux-gnu
```

This should require upwards of 500 Mb of free system memory, effectively exceeding the total RAM of a Pi Zero 2W.

Both `cargo cross build` and `cargo zigbuild` default to compiling with the `--profile=release` flag, applying some optimizations and considerably lowering the resulting binary file size as compared to when building with `--profile=dev`.

```sh
rsync -avz --progress target/aarch64-unknown-linux-gnu/release/wg_monitor user@pi:~/
```

Replace `release` with `debug` to transfer the binary of a `--profile=dev` build.

### `-j1`

You *may* have some luck building it on the Pi if you build it in a serial mode, compiling one dependency at a time. Swap is probably still required.

```sh
cargo build -j1
```

Mind that build times will be *very* long. Cross-compilation is recommended. Failing that, remember to at least use a heatsink.

## configuration

Configuration is done by modifying files created in the *configuration directory*, which is one of the following locations, in decreasing order of precedence:

* ...as was explicitly declared with `--config-dir=/path/to/directory`
* `$WG_MONITOR_CONFIG_DIR` if set
* `/etc/wg_monitor` if your user is root
* `$XDG_CONFIG_HOME/wg_monitor` if `$XDG_CONFIG_HOME` is set
* `$HOME/.config/wg_monitor`
* fail if `$HOME` is unset

Running the program with `--save` will create this directory, including parent directories if necessary.

```sh
cargo run -- --save
```

Mind that the program will likely require to be run with root permissions to be able to issue queries for handshake timestamps of the WireGuard interface. As per the list above, running as root would make the configuration directory default to `/etc/wg_monitor`.

### `config.toml`

As part of `--save`, a new `config.toml` will be created in the configuration directory, if it does not already exist. Edit it like you would any text file. `--save` will not overwrite an existing `config.toml`, but beware that any comments will be removed.

### `peers.txt`

A new `peers.txt` file will also have been created in the configuration diretcory, next to the `config.toml` file. Complete it with the public keys of the peers you want to monitor. You can make it easier to distinguish between peers by appending a human-readable name after each key, separated by whitespace (such as a space or a tab).

Lines that start with an octothorpe `#` are ignored.

```text
# <public key> <description>
CrfE/XA7bVuTv2OVM3wzD2PeHw7EldvkCB8tkdq1Oi2= Alice's house
XAigmEW/rc0fVvSsnw0xyzElf1vmtFbAe9w7cz+BXg7= Bob's apartment
#Wd03M/v1Q7pcGHlfm7nMB4KV/2As9yi5KxSgn9Qa6xl= Eve's cottage
```

## backends

There are three available notification backends.

### slack

Messages to Slack channels can trivially be pushed by use of [webhook URLs](https://en.wikipedia.org/wiki/Webhook). HTTP requests made to these will end up as messages in the channels they refer to. See [this guide](https://docs.slack.dev/messaging/sending-messages-using-incoming-webhooks) in the Slack documentation for developers on how to get started.

It is recommended that you make an entry in `/etc/hosts` to manually resolve `hooks.slack.com` to *an* IP of the underlying Slack server, to avoid potential DNS lookup failures.

URLs must be be quoted. You may enter any number of URLs as long as you separate the individual strings with a comma.

```toml
[slack]
enabled = true
urls = ["https://hooks.slack.com/services/REDACTED/ALSOTHIS/asdfasdfasdf", "https://hooks.slack.com/services/ASDFASDF/FDSAFDSA/qwertyiiioqer"]
```

#### formatting messages

Slack supports some formatting. Text between asterisks `*` will be in \***bold**\*, text between underscores `_` will be in \_*italics*\_, text between tildes `~` will be in \~~~strikethrough~~\~, etc.

Strings defined in the configuration file can make use of this.

```toml
[slack.strings]
header = ""
first_run_header = ":zap: *Power restored* _(or restart of device)_"
bullet_point = " *-* "
```

See [this help article](https://slack.com/intl/en-gb/help/articles/360039953113-Format-your-messages-in-Slack-with-markup) for the full listing.

### batsign

[**Batsign**](https://batsign.me) is a free (gratis) service with which you can send brief emails. Requires registration, after which you will receive a unique URL that should be kept secret. HTTP requests made to this URL will send an email to the address you specified when registering.

It is recommended that you make an entry in `/etc/hosts` to manually resolve `batsign.me` to the IP of the underlying Batsign server, to avoid potential DNS lookup failures.

URLs must be quoted. You may enter any number of URLs as long as you separate the individual strings with a comma.

```toml
[batsign]
enabled = true
urls = ["https://batsign.me/at/example@address.tld/asdfasdf", "https://batsign.me/at/other@address.tld/fdsafdafa"]
```

#### formatting mails

It is not possible to format text in Batsign emails with HTML markup. The best you can do is to use Unicode characters.

### external command

You can also have the program execute an external command as a way to push notifications, although there are several caveats.

* The command run will be passed several arguments in a specific hardcoded order, and it is unlikely that it will immediately suit whatever notification program you want to use. Realistically what you will end up doing is writing some glue-layer script that maps the arguments to something the notification program can use.

* If you run the project binary as root (which may well be unavoidable) the external command specified will in turn also be run as root. If you need it to be run as a different user, you will have to use `systemd-run` or `su` in your shell script.

In the configuration file;

```toml
[command]
enabled = true
commands = ["/absolute/path/to/script.sh"]
```

Remember to `chmod` the script executable `+x`.

#### arguments

The order of arguments is as follows:

1. The composed message body, formatted with strings as defined in the configuration file
2. The path to the `peers.txt` file
3. The number of time the main loop has run (starting at 0, unless `--resume` was passed, in which case it starts at 1)
4. A comma-separated string of lost keys in the format "`key:timestamp`"
5. A comma-separated string of missing keys in the format "`key:timestamp`"
6. A comma-separated string of keys that were lost the previous loop in the format "`key:timestamp`"
7. A comma-separated string of keys that were missing the previous loop in the format "`key:timestamp`"
8. In non-reminder notifications, a comma-separated string of keys that are now lost in the format "`key:timestamp`"
9. In non-reminder notifications, a comma-separated string of keys that are now missing in the format "`key:timestamp`"
10. In non-reminder notifications, a comma-separated string of keys that were lost (but are no longer) in the format "`key:timestamp`"
11. In non-reminder notifications, a comma-separated string of keys that were missing (but are no longer) in the format "`key:timestamp`"

Any parameter for which there is no value (as in, there are no lost peers so there are no lost keys), the argument is passed but is simply an empty string `""`.

#### example scripts

[`notify-send`](https://man.archlinux.org/man/notify-send.1) can be used to send desktop notifications. Here are some example glue-layer scripts that map the arguments passed by the external command backend into something `notify-send` can work with.

##### [`notify-send-to-all-gui.sh`](notify-send-to-all-gui.sh)

This will push a desktop notification to *all* users currently logged into a graphical environment on the current machine.

Adapted from [the example on the Arch Linux wiki](https://wiki.archlinux.org/title/Desktop_notifications#Send_notifications_to_all_graphical_users):

```bash
#!/bin/bash

# $1 contains the composed message
# $2 contains the path to the peers.txt file, not relevant for this script
# $3 contains the loop iteration number

title="WireGuard Monitor"
icon="network-wireless-disconnected"
urgency="critical"
loop_number="$3"
message="$1"

ids=( $(loginctl list-sessions -j | jq -r '.[] | .session') )

if [[ $loop_number = 0 ]]; then
    # run 0
    summary="$title: first run"
else
    summary="$title: update"
fi

for id in "${ids[@]}" ; do
    [[ $(loginctl show-session $id --property=Type) =~ (wayland|x11) ]] || continue

    user=$(loginctl show-session $id --property=Name --value)

    systemd-run --machine=${user}@.host --user \
        notify-send \
            --app-name="$title" \
            --icon="$icon" \
            --urgency="$urgency" \
            "$summary" \
            "$message"
done
```

##### [`notify-send-to-one.sh`](notify-send-to-one.sh)

A similar script but for only *one* user. Change the `user=` line to match the user that should receive the notification. It accepts both usernames and user IDs.

```bash
#!/bin/bash

# $1 contains the composed message
# $2 contains the path to the peers.txt file, not relevant for this script
# $3 contains the loop iteration number

# make sure to change the "user" variable to the actual username or user ID
# of the user you want to send the notification to, e.g. 1000, "bob" or "alice".

user=1000

title="WireGuard Monitor"
icon="network-wireless-disconnected"
urgency="critical"
loop_number="$3"
message="$1"

if [[ $loop_number = 0 ]]; then
    # run 0
    summary="$title: first run"
else
    summary="$title: update"
fi

systemd-run --machine=${user}@.host --user \
    notify-send \
        --app-name="$title" \
        --icon="$icon" \
        --urgency="$urgency" \
        "$summary" \
        "$message"
```

## systemd

The program is preferably run as a [**systemd**](https://systemd.io) service, to have it be automatically restarted upon restoration of power. To facilitate this, [a service unit file](wg_monitor.service) is provided in the repository.

It will have to be copied (or symlinked) into `/etc/systemd/system`, after which you can use `systemctl edit` to create a drop-in file that overrides the `ExecStart` directive in the unit file to point to the actual location of the `wg_monitor` binary. This is not required if the binary is already located in the default path of `/usr/local/bin/wg_monitor`.

You can find the binaries you compile with Cargo in the `target/<profile>/` subdirectory of the project, where `<profile>` is either `debug` or `release`, depending on what profile you built with.

```sh
sudo cp wg_monitor.service /etc/systemd/system
sudo systemctl edit wg_monitor.service
```

```ini
### Editing /etc/systemd/system/wg_monitor.service.d/override.conf
### Anything between here and the comment below will become the contents of the drop-in file

[Service]
ExecStart=
ExecStart=/home/user/src/wg_monitor/wg_monitor --disable-timestamps --verbose

### Edits below this comment will be discarded
### ...
```

An empty `ExecStart=` must be used to clear the value set in the original file, as `Exec` directives are additive.

```sh
sudo systemctl daemon-reload
sudo systemctl enable --now wg_monitor.service
```

`enable --now` both enables the service to be autostarted on subsequent boots as well as starts it immediately. For the terminal output of the program (and error messages if it could not be started), refer to the systemd [**journal**](https://wiki.archlinux.org/title/Systemd/Journal).

```sh
journalctl -b0 -fn100 -u wg_monitor.service
```

### starts too early

The systemd service is set up to start only after networking has been set up, but it might still take a while for peers to connect and handshake. If the program starts before peers have done this, it will report them as missing.

To mitigate this, you can use the `--sleep` flag to have the program wait for a specified duration before starting the monitoring loop. The flag takes a human-readable duration string as argument, like `10s`, `1m`, `2h` and so forth.

```ini
[Service]
ExecStart=
ExecStart=/home/user/src/wg_monitor/wg_monitor --disable-timestamps --verbose --sleep 5m
```

## ai

[**GitHub Copilot AI**](https://github.com/features/copilot/ai-code-editor) was used (in [**Visual Studio Code**](https://code.visualstudio.com)) for inline suggestions and to tab-complete some code and documentation. [**ChatGPT**](https://chatgpt.com) and [**Claude**](https://claude.ai) were used to answer questions and teach Rust. No code from "write me a function doing *xyz*" prompts is included in this project.

## todo

* currently nothing, ideas welcome

## license

This project is dual-licensed under the [**MIT License**](LICENSE-MIT) and the [**Apache License (Version 2.0)**](LICENSE-APACHE) at your option.
