# hna-list-rs

`hna-list-rs` is a tool for showing OLSR Host-Network-Announcements (HNA) in a nicely formatted way. I programmed it primarily for using it in the Freifunk Berlin network, but it might work in other olsrd(v1) network as well.

This tool makes a strong assumption on being run on OpenWrt, which especially gets reflected in the hard-coded paths for the data sources.

In addition, it requires an enabled json-plugin on the olsr-daemon.

## Building for OpenWrt

Unfortunately there is currently no stable rust-support in OpenWrt (as of January 2023). To get it running on OpenWrt anyway, one needs to cross-compile is.

With the rust-tool [cross](https://github.com/cross-rs/cross) this is not a too big deal anymore. For an archer-c5-v1 (which uses mips_24kc architecture), a compilation call looks like this:

```sh
cross build --release --target mips-unknown-linux-musl
```

The resulting binary can be copied to the router and used thereafter.

For running on recent [falter](https://github.com/freifunk-berlin/falter-packages) (by the time of writing falter-1.2.2) there needs nothing more to be done.