# Rusty Realm

<p align="center">
  <img src="assets/rusty-realm-banner.svg" width="100%" alt="Rusty Realm ASCII-art banner">
</p>

<p align="center"><em>A cross-platform Rust revival of <a href="https://finalrealm.weebly.com/">Final Realm</a>, the original Windows-exclusive Batch game.</em></p>

Rusty Realm is a Rust-based, cross-platform revival of [Final Realm](https://finalrealm.weebly.com/), a text-driven game originally built for Windows using Batch. The project rebuilds its game systems in Rust, with a portable codebase and a more robust foundation for continued development.

The current codebase includes character accounts and save data, combat, quests, skills, inventory, shops, banking, and the Stronghold challenge. It is a work in progress; see the source code for the current implementation.

## The original game

Visit [Final Realm's original website](https://finalrealm.weebly.com/) for the game that inspired this project.

## Development

This project is written in Rust (2021 edition). To build and test it locally:

```sh
cargo build
cargo test
```
