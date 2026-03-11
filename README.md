# wg_monitor

Monitors other peers in a [Wireguard VPN](https://www.wireguard.com) and sends a notification if contact with a peer is lost.

The main purpose of this is to monitor Internet-connected locations for power outages, using Wireguard handshakes as a way for sites to phone home. Each site needs an always-on, always-connected computer to act as a Wireguard peer, for which something like a [Raspberry Pi Zero 2W](https://www.raspberrypi.com/products/raspberry-pi-zero-2-w) is cheap and more than sufficient. ([Cross-compilation may be required.](#cross-compilation))

In a hub-and-spoke Wireguard configuration, this should be run on the hub server, with an additional instance on at least one other *geographically disconnected* peer to monitor the hub. In other configurations, it can be run on any peer with visibility of other peers, but a secondary instance monitoring the first is recommended in any setup. If the hub loses power, it cannot report itself as being lost.

Peers must have a `PersistentKeepalive` setting in their Wireguard configuration with a value *comfortably lower* than the peer timeout of this program. This timeout is **10 minutes** by default.

Notifications are sent as [**Slack** webhook messages](https://docs.slack.dev/messaging/sending-messages-using-incoming-webhooks) and/or as short emails via [**Batsign**](https://batsign.me).

## usage

```
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

## config.toml

Changing settings is done by editing a configuration file. You can generate a new one by passing `--save`.

This `config.toml` file will be placed in somewhere of the following locations, in decreasing order of precedence:

* ...as was overridden with `--config-dir=/path/to/directory`
* `/etc/wg_config` **if** your user is root
* `$XDG_CONFIG_HOME/wg_config` if `$XDG_CONFIG_HOME` is set
* `$HOME/.config/wg_config`

Running the program with `--save` will not overwrite previous contents in an existing file, but beware that any comments will be removed.

## peers

A `peers.txt` file will have been created next to the configuration `config.toml` file. Complete it with the public keys of the peers you want to monitor. You can attach a human-readable name to the key if you separate them with a normal space. Lines that start with an octothorpe `#` will be ignored by the program.

```
# <public key> <description>
CrfE/XA7bVuTv2OVM3wzD2PeHw7EldvkCB8tkdq1Oi2= Alice's house
AigmEW/rc0fVvSsnw0xyzElf1vmtFbAe9w7cz+BXg7= Bob's apartment
#Wd03M/v1Q7pcGHlfm7nMB4KV/2As9yi5KxSgn9Qa6xl= Eve's cottage
```

## compilation

This project uses [**Cargo**](https://doc.rust-lang.org/cargo) for compilation and dependency management.

```
$ cargo build
$ cargo run -- --help
$ cargo run -- --save
```

## cross-compilation

A Pi can trivially *run* the program, but does not have enough memory to compile it, at least not with the default flags. You can probably still build it by adding swap and exercising a lot of patience, but the convenient way is to just cross-compile it on another Linux computer and transfering the resulting binary.

> Mind that your `$CFLAGS` environment variable must not contain `-march=native` for all dependencies to build.

```
$ export CFLAGS="-O2 -pipe"  # as an example
$ cargo build --target=aarch64-unknown-linux-gnu
$ rsync -avz --progress target/aarch64-unknown-linux-gnu/debug/wg_monitor user@pi:~/
```

This should require upwards of 600 Mb of free system memory, exceeding the total RAM of the Pi Zero 2W.

Add `--release` to build the project in release mode, applying some optimizations and considerably lowering the binary file size.

### `-j1`

All that said, you *may* have some luck building it on the Pi if you build it in serial mode, compiling one dependency at a time.

```
$ cargo build --release -j1
```

Mind that build times will be *very* long. Cross-compilation is recommended.

### gcc target packages

If you cannot install the required packages for **AArch64** cross-compilation, such as if you are running an immutable distro (like [**Aurora**](https://getaurora.dev) or [**Bazzite**](https://bazzite.gg)), consider compiling it from within a [**Distrobox** container](https://wiki.archlinux.org/title/Distrobox). There are graphical container managers available as Flatpaks, such as [**Kontainer**](https://flathub.org/apps/io.github.DenysMb.Kontainer) and [**Distroshelf**](https://flathub.org/en/apps/com.ranfdev.DistroShelf), that can facilitate fetching and installing images. As of the time of writing and on a system running Aurora, the `ghcr.io/ublue-os/arch-distrobox:latest` Arch Linux image works very well.

```
$ sudo pacman -S rust-aarch64-gnu
```

A pre-compiled binary will be provided under [**Releases**](https://github.com/zorael/wg_monitor/releases) once the code stabilizes a bit and `v1.0.0` can be tagged.

## todo

* external command as notification method
* better documentation
* colored terminal output?
* pre-compiled binary

## license

This project is dual-licensed under the [**MIT License**](LICENSE-MIT) and the [**Apache License (Version 2.0)**](LICENSE-APACHE) at your option.
