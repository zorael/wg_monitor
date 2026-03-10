# wg_monitor

Monitors other peers in a [Wireguard VPN](https://www.wireguard.com) and sends a notification if contact with a peer is lost.

The main purpose of this is to monitor Internet-connected locations for power outages, using Wireguard handshakes as a way for sites to phone home. Each needs an always-on, always-connected computer to act as a Wireguard peer, for which something like a [Raspberry Pi Zero 2W](https://www.raspberrypi.com/products/raspberry-pi-zero-2-w) is cheap and more than sufficient.

In a hub-and-spoke Wireguard configuration, this should be run on the hub server, ideally with an additional instance on (at least) one other geographically disconnected peer to monitor the hub. In other configurations, it can be run on any peer with visibility of other peers, but a secondary instance monitoring the first is recommended in any setup.

Peers must have a `PersistentKeepalive` setting in their Wireguard configuration with a value *comfortably lower* than the peer timeout of this program. This timeout is **600 seconds** by default, but can be overridden by modifying a configuration file.

Notifications are sent as [**Slack** webhook messages](https://docs.slack.dev/messaging/sending-messages-using-incoming-webhooks) and/or as short emails via [**Batsign**](https://batsign.me).

## usage

```
wg_monitor x.y.z | copyright 2026 jr
$ git clone https://github.com/zorael/wg_monitor

Usage: wg_monitor [OPTIONS]

Options:
  -c, --config-dir <path>  Specify an alternate configuration directory
      --resume             Skip notifications about program startup
      --skip-first         Skip the first run and thus the first notification
      --show               Show the resolved configuration and exit
  -v, --verbose            Print some additional information
  -d, --debug              Print additional debug information
      --dry-run            Perform a dry run without sending any notifications
      --save               Write configuration to disk
```

## config.toml

Configuration is largely done by editing a configuration file. Supply `--save` to generate a new one. It will be created someplace contextual to your user, creating directories as necessary.

* `/home/user/.config/wg_monitor/config.toml` for normal users (directory overridden by `$XDG_CONFIG_HOME`)
* `/etc/wg_monitor/config.toml` for root

Pass `--config-dir=/path/to/somewhere/else` to override the target location.

## todo

* external command as notification method
* better documentation
* colored terminal output?

## license

This project is dual-licensed under the [**MIT License**](LICENSE-MIT) and the [**Apache License (Version 2.0)**](LICENSE-APACHE) at your option.
