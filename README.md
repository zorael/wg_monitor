# wg_monitor

Monitors other peers in a [**Wireguard** VPN](https://www.wireguard.com) and sends a notification if contact with a peer is lost.

The main purpose of this is to monitor Internet-connected locations for power outages, using Wireguard handshakes as a way for sites to phone home. Each site needs an always-on, always-connected computer to act as a Wireguard peer, for which something like a [**Raspberry Pi Zero 2W**](https://www.raspberrypi.com/products/raspberry-pi-zero-2-w) is cheap and more than sufficient. ([Cross-compilation](#cross-compilation) may be required.)

In a hub-and-spoke Wireguard configuration, this should be run on the hub server, with an additional instance on at least one other *geographically disconnected* peer to monitor the hub. In other configurations, it can be run on any peer with visibility of other peers, but a secondary instance monitoring the first is recommended in any setup. If the hub loses power, it cannot report itself as being lost.

Peers must have a `PersistentKeepalive` setting in their Wireguard configuration with a value *comfortably lower* than the peer timeout of this program. This timeout is **10 minutes** by default.

Notifications are sent as [**Slack** webhook messages](https://docs.slack.dev/messaging/sending-messages-using-incoming-webhooks), as short emails via [**Batsign**](https://batsign.me), and/or by invocation of [an external command](#external-command).

## tl;dr

```text
wg_monitor x.y.z | copyright 2026 jr
$ git clone https://github.com/zorael/wg_monitor

Usage: wg_monitor [OPTIONS]

Options:
  -c, --config-dir <path>  Specify an alternate configuration directory
      --resume             Word the first notification as if the program was not just started
      --skip-first         Skip the first run and thus the first notification
      --show               Output configuration to screen and exit
  -v, --verbose            Print some additional information
  -d, --debug              Print much more additional information
      --dry-run            Perform a dry run, echoing what would be done
      --save               Write configuration to disk
```

To get started, create new [configuration](#configtoml) and [peer list](#peers) files by passing `--save`.

```sh
cargo run -- --save
```

## toc

* [compilation](#compilation)
  * [cross-compilation](#cross-compilation)
  * [-j1](#-j1)
* [config.toml](#configtoml)
* [peers](#peers)
* [slack](#slack)
  * [formatting](#formatting)
* [batsign](#batsign)
* [external command](#external-command)
  * [arguments](#arguments)
  * [example](#example)
* [todo](#todo)
* [license](#license)

---

## compilation

This project uses [**Cargo**](https://doc.rust-lang.org/cargo) for compilation and dependency management.

```sh
cargo build
cargo run -- --help
cargo run -- --save
```

A normal desktop or laptop computer should be able to trivially build it without any additional steps taken.

Pre-compiled binaries will be provided under [**Releases**](https://github.com/zorael/wg_monitor/releases) once the code stabilizes a bit and `v1.0.0` can be tagged.

### cross-compilation

A device like the Pi Zero 2W can *run* the program but does not have enough memory to compile it, at least not with default flags. You can probably still build it by adding swap and exercising a lot of patience, but the convenient way is to just cross-compile it on another (Linux) computer and transferring the resulting binary.

Regrettably, manually setting up cross-compilation can be non-trivial. As such, use of one of `cargo-cross` and `cargo-zigbuild` is recommended (but not required). For the latter you need to install a [**Zig**](https://ziglang.org) compiler. Refer to your repositories, alternatively install it via Homebrew (`brew install zig`).

Note that your `$CFLAGS` environment variable must not contain `-march=native` for all dependencies to successfully build.

```sh
cargo install cargo-cross
CFLAGS="-O2 -pipe" cargo cross build --target=aarch64-unknown-linux-gnu
```

```sh
cargo install cargo-zigbuild
CFLAGS="-O2 -pipe" cargo zigbuild --target=aarch64-unknown-linux-gnu
```

```sh
rsync -avz --progress target/aarch64-unknown-linux-gnu/release/wg_monitor user@pi:~/
```

This should require upwards of 500 Mb of free system memory, effectively exceeding the total RAM of the Pi Zero 2W.

Both `cargo cross build` and `cargo zigbuild` default to compiling with the `--profile=release` flag, applying some optimizations and considerably lowering the resulting binary file size as compared to when building with `--profile=dev`.

### `-j1`

You *may* have some luck building it on the Pi if you build it in a serial mode, compiling one dependency at a time. Swap is probably still required.

```sh
cargo build --release -j1
```

Mind that build times will be *very* long. Cross-compilation is recommended. Otherwise, remember to use a heatsink.

## `config.toml`

Changing settings is done by editing a configuration file. You can generate a new one by passing `--save`.

A new `config.toml` file will be created in one of the following locations, in decreasing order of precedence:

* ...as was explicitly declared with `--config-dir=/path/to/directory`
* `$WG_MONITOR_CONFIG_DIR` if set
* `/etc/wg_monitor` if your user is root
* `$XDG_CONFIG_HOME/wg_monitor` if `$XDG_CONFIG_HOME` is set
* `$HOME/.config/wg_monitor`
* fail if `$HOME` is unset

The program will likely require root permissions to be able to issue queries for handshake timestamps of the Wireguard interface. Mind that, as per the list above, this would make the configuration directory default to `/etc/wg_monitor`.

Directories will be created as necessary, including parent directories.

Running the program with `--save` will not overwrite previous contents in an existing file, but beware that any comments will be removed.

## peers

A new `peers.txt` file will have been created next to the configuration `config.toml` file. Complete it with the public keys of the peers you want to monitor. You can make it easier to distinguish between peers by appending a human-readable name after each key, separated by a normal space character.

Lines that start with an octothorpe `#` will be ignored.

```text
# <public key> <description>
CrfE/XA7bVuTv2OVM3wzD2PeHw7EldvkCB8tkdq1Oi2= Alice's house
XAigmEW/rc0fVvSsnw0xyzElf1vmtFbAe9w7cz+BXg7= Bob's apartment
#Wd03M/v1Q7pcGHlfm7nMB4KV/2As9yi5KxSgn9Qa6xl= Eve's cottage
```

## slack

Messages to Slack channels can trivially be pushed by enabling one or more webhook URLs. HTTP POST requests made to those URLs will end up as messages in their respective channels. See [this guide](https://docs.slack.dev/messaging/sending-messages-using-incoming-webhooks) in the Slack documentation for developers on how to get started.

You may enter any number of urls as long as you separate the individual strings with a comma.

```toml
[slack]
enabled = true
urls = ["https://hooks.slack.com/services/REDACTED/ALSOTHIS/asdfasdfasdf", "https://hooks.slack.com/services/ASDFASDF/FDSAFDSA/qwertyiiioqer"]
```

### formatting

Slack supports some formatting. Text between \***two asterisks**\* will be in **bold**, text between \_*two underscores*\_ will be in *italics*, text between \~~~two tildes~~\~ will be in ~~strikethrough~~, etc.

Strings defined in the configuration file can make use of this.

```toml
[slack.strings]
header = ""
first_run_header = ":zap: *Power restored* _(or restart of device)_"
bullet_point = "*-* "
```

See [this help article](https://slack.com/intl/en-gb/help/articles/360039953113-Format-your-messages-in-Slack-with-markup) for the full listing.

## batsign

It is likewise easy to push simple email notifications by signing up for a [Batsign](https://batsign.me) address. Much like Slack webhooks, HTTP POST requests made to the URL you receive will end up as emails sent to the corresponding addresses.

```toml
[batsign]
enabled = true
urls = ["https://batsign.me/at/example@address.tld/asdfasdf", "https://batsign.me/at/other@address.tld/fdsafdafa"]
```

## external command

It is possible to have the program execute an external command to push notifications, although there are several caveats.

* The command run will be passed several arguments in a specific order, and it is unlikely that it will immediately suit whatever notification program you want to use. Realistically what you will end up doing is writing some glue-layer script that maps the arguments to something you can use.

* If you run the project binary as root (which may be unavoidable) the command it runs will also be run as root. If you need it to be run as your own user, you will have to use `su` in your shell script, and even then environment variables may prove a problem.

### arguments

The order of arguments is as follows:

1. The composed message body, formatted with strings as defined in the configuration file
2. The path to the `peers.txt` file
3. The number of time the notification loop has run (starting at 0)
4. A comma-separated string of late keys
5. A comma-separated string of missing keys
6. A comma-separated string of keys that were late the previous loop
7. A comma-separated string of keys that were missing the previous loop
8. In non-reminder notifications, a comma-separated string of keys that became late
9. In non-reminder notifications, a comma-separated string of keys that went missing
10. In non-reminder notifications, a comma-separated string of keys that are no longer late
11. In non-reminder notifications, a comma-separated string of keys that returned

Any parameter for which there is no value (as in, there are no late peers so there are no late keys), the argument is passed but is simply empty.

### (untested) example

```bash
#!/bin/bash

icon="network-wireless-disconnected"
urgency="critical"

if [[ "$3" = "0" ]]; then
    # loop iteration 0
    summary="Wireguard Monitor: first run"
else
    summary="Wireguard Monitor: update"
fi

notify-send \
    --icon="$icon" \
    --urgency="$urgency" \
    "$summary"
    "$1"
```

In the configuration file;

```toml
[command]
enabled = true
commands = ["/absolute/path/to/script.sh"]
```

## todo

* better documentation
* colored terminal output?
* pre-compiled binary

## license

This project is dual-licensed under the [**MIT License**](LICENSE-MIT) and the [**Apache License (Version 2.0)**](LICENSE-APACHE) at your option.
