# wg_monitor

Monitors other peers in a [**WireGuard**](https://www.wireguard.com) VPN and sends a notification if contact with a peer is lost.

The main purpose of this is to monitor Internet-connected locations for power outages, using WireGuard handshakes as a way for sites to phone home. Each site needs an always-on, always-online computer to act as a WireGuard peer, for which something like a [**Raspberry Pi Zero 2W**](https://www.raspberrypi.com/products/raspberry-pi-zero-2-w) is cheap and more than sufficient. ([Cross-compilation](#cross-compilation) may be required.)

In a hub-and-spoke WireGuard configuration, this program should be run on the hub server, with an additional instance on at least one other *geographically disconnected* peer to monitor the hub. In other configurations, it can be run on any peer with visibility of other peers, but a secondary instance monitoring the first is recommended in any setup. If the hub loses power, it cannot report itself as being lost.

Peers must have a `PersistentKeepalive` setting in their WireGuard configuration with a value *comfortably lower* than the peer timeout of this program. This timeout is **10 minutes** by default.

Notifications can be sent as [**Slack**](https://docs.slack.dev/messaging/sending-messages-using-incoming-webhooks) messages, as short emails via [**Batsign**](https://batsign.me), and/or by invocation of an [**external command**](#external-command) (like `notify-send`).

## tl;dr

```text
wg_monitor x.y.z | copyright 2026 jr
$ git clone https://github.com/zorael/wg_monitor

Usage: wg_monitor [OPTIONS]

Options:
  -c, --config-dir <path>   Specify an alternate configuration directory
      --resume              Word the first notification as if the program was not just started
      --skip-first          Skip the first run and thus the first notification
      --disable-timestamps  Disable timestamps in terminal output
      --show                Output configuration to screen and exit
  -v, --verbose             Print some additional information
  -d, --debug               Print much more additional information
      --dry-run             Perform a dry run, echoing what would be done
      --save                Write configuration to disk
  -V, --version             Display version information and exit
```

To get started, create new [configuration](#configtoml) and [peer list](#peerstxt) files by passing `--save`.

```sh
cargo run -- --save
```

## toc

* [compilation](#compilation)
  * [cross-compilation](#cross-compilation)
  * [`-j1`](#-j1)
* [`config.toml`](#configtoml)
* [`peers.txt`](#peerstxt)
* [slack](#slack)
  * [formatting](#formatting)
* [batsign](#batsign)
* [external command](#external-command)
  * [arguments](#arguments)
  * [example scripts](#example-scripts)
    * [notify all](#notify-all)
    * [notify one](#notify-one)
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

## `config.toml`

Changing settings is done by editing a configuration file. You can generate a new one by passing `--save`.

A new `config.toml` file will be created in one of the following locations, in decreasing order of precedence:

* ...as was explicitly declared with `--config-dir=/path/to/directory`
* `$WG_MONITOR_CONFIG_DIR` if set
* `/etc/wg_monitor` if your user is root
* `$XDG_CONFIG_HOME/wg_monitor` if `$XDG_CONFIG_HOME` is set
* `$HOME/.config/wg_monitor`
* fail if `$HOME` is unset

The program will likely require root permissions to be able to issue queries for handshake timestamps of the WireGuard interface. Mind that, as per the list above, this would make the configuration directory default to `/etc/wg_monitor`.

Directories will be created as necessary, including parent directories.

Running the program with `--save` will not overwrite previous contents in an existing file, but beware that any comments will be removed.

## `peers.txt`

A new `peers.txt` file will also have been created in the configuration diretcory, next to the `config.toml` file. Complete it with the public keys of the peers you want to monitor. You can make it easier to distinguish between peers by appending a human-readable name after each key, separated by a normal space character.

Lines that start with an octothorpe `#` will be ignored.

```text
# <public key> <description>
CrfE/XA7bVuTv2OVM3wzD2PeHw7EldvkCB8tkdq1Oi2= Alice's house
XAigmEW/rc0fVvSsnw0xyzElf1vmtFbAe9w7cz+BXg7= Bob's apartment
#Wd03M/v1Q7pcGHlfm7nMB4KV/2As9yi5KxSgn9Qa6xl= Eve's cottage
```

## slack

Messages to Slack channels can trivially be pushed by use of [webhook URLs](https://en.wikipedia.org/wiki/Webhook). HTTP requests made to these will end up as messages in the channels they refer to. See [this guide](https://docs.slack.dev/messaging/sending-messages-using-incoming-webhooks) in the Slack documentation for developers on how to get started.

URLs must be be quoted. You may enter any number of URLs as long as you separate the individual strings with a comma.

```toml
[slack]
enabled = true
urls = ["https://hooks.slack.com/services/REDACTED/ALSOTHIS/asdfasdfasdf", "https://hooks.slack.com/services/ASDFASDF/FDSAFDSA/qwertyiiioqer"]
```

### formatting

Slack supports some formatting. Text between asterisks `*` will be in \***bold**\*, text between underscores `_` will be in \_*italics*\_, text between tildes `~` will be in \~~~strikethrough~~\~, etc.

Strings defined in the configuration file can make use of this.

```toml
[slack.strings]
header = ""
first_run_header = ":zap: *Power restored* _(or restart of device)_"
bullet_point = " *-* "
```

See [this help article](https://slack.com/intl/en-gb/help/articles/360039953113-Format-your-messages-in-Slack-with-markup) for the full listing.

## batsign

It is likewise easy to push simple email notifications by signing up for a [**Batsign**](https://batsign.me) address. Much like Slack webhooks, HTTP requests made to these will end up as emails sent to the corresponding addresses.

URLs must be quoted. You may enter any number of URLs as long as you separate the individual strings with a comma.

```toml
[batsign]
enabled = true
urls = ["https://batsign.me/at/example@address.tld/asdfasdf", "https://batsign.me/at/other@address.tld/fdsafdafa"]
```

## external command

You can also have the program execute an external command as a way to push notifications, although there are several caveats.

* The command run will be passed several arguments in a specific hardcoded order, and it is unlikely that it will immediately suit whatever notification program you want to use. Realistically what you will end up doing is writing some glue-layer script that maps the arguments to something you can use.

* If you run the project binary as root (which may be unavoidable) the external command you set up as notification command will in turn also be run as root. If you need it to be run as a different user, you will have to use `systemd-run` or `su` in your shell script.

### arguments

The order of arguments is as follows:

1. The composed message body, formatted with strings as defined in the configuration file
2. The path to the `peers.txt` file
3. The number of time the notification loop has run (starting at 0, unless `--resume` was passed, in which case it starts at 1)
4. A comma-separated string of late keys in the format "`key:timestamp`"
5. A comma-separated string of missing keys in the format "`key:timestamp`"
6. A comma-separated string of keys that were late the previous loop in the format "`key:timestamp`"
7. A comma-separated string of keys that were missing the previous loop in the format "`key:timestamp`"
8. In non-reminder notifications, a comma-separated string of keys that became late in the format "`key:timestamp`"
9. In non-reminder notifications, a comma-separated string of keys that went missing in the format "`key:timestamp`"
10. In non-reminder notifications, a comma-separated string of keys that are no longer late in the format "`key:timestamp`"
11. In non-reminder notifications, a comma-separated string of keys that returned in the format "`key:timestamp`"

Any parameter for which there is no value (as in, there are no late peers so there are no late keys), the argument is passed but is simply an empty string.

### example scripts

This should theoretically push a desktop notification to all users currently logged into a graphical environment, leveraging `notify-send` for the actual notification.

#### notify all

Example [`notify-send-to-all-gui.sh`](notify-send-to-all-gui.sh), adapted from [the example on the Arch Linux wiki](https://wiki.archlinux.org/title/Desktop_notifications#Send_notifications_to_all_graphical_users):

```bash
#!/bin/bash

# $1 contains the composed message
# $2 contains the path to the peers.txt file, not relevant for this script
# $3 contains the loop iteration number

icon="network-wireless-disconnected"
urgency="critical"
loop_number=$3
ids=( $(loginctl list-sessions -j | jq -r '.[] | .session') )

if [[ "$loop_number" = "0" ]]; then
    # run 0
    summary="WireGuard Monitor: first run"
else
    summary="WireGuard Monitor: update"
fi

for id in "${ids[@]}" ; do
    [[ $(loginctl show-session $id --property=Type) =~ (wayland|x11) ]] || continue

    user=$(loginctl show-session $id --property=Name --value)

    systemd-run --machine=${user}@.host --user \
        notify-send \
            --icon="$icon" \
            --urgency="$urgency" \
            "$summary" \
            "$1"
done
```

#### notify one

Example [`notify-send-to-one`](notify-send-to-one.sh):

```bash
#!/bin/bash

# $1 contains the composed message
# $2 contains the path to the peers.txt file, not relevant for this script
# $3 contains the loop iteration number

# make sure to change the "user" variable to the actual username or user ID
# of the user you want to send the notification to, e.g. 1000, "bob" or "alice".

icon="network-wireless-disconnected"
urgency="critical"
loop_number=$3
user=1000

if [[ "$loop_number" = "0" ]]; then
    # run 0
    summary="WireGuard Monitor: first run"
else
    summary="WireGuard Monitor: update"
fi

systemd-run --machine=${user}@.host --user \
    notify-send \
        --icon="$icon" \
        --urgency="$urgency" \
        "$summary" \
        "$1"
```

In the configuration file;

```toml
[command]
enabled = true
commands = ["/absolute/path/to/script.sh"]
```

Remember to `chmod` it executable `+x`.

## todo

* better documentation
* colored terminal output?
* pre-compiled binary

## license

This project is dual-licensed under the [**MIT License**](LICENSE-MIT) and the [**Apache License (Version 2.0)**](LICENSE-APACHE) at your option.
