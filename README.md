[![wakatime](https://wakatime.com/badge/github/angeloanan/hydrate-reminder.svg?style=flat)](https://wakatime.com/badge/github/angeloanan/hydrate-reminder)

# Hydrate Reminder

A smol & simple reminder app to remind you to drink water every hour you didn't drink.

This is a personal project of mine to explore building a native desktop app using [Tauri](https://tauri.studio/) and "refresh" myself on Rust development. 

Contributions are welcome, though not expected and not guaranteed to be merged; this is a personal project after all.

Save data are stored in a [packed serialized binary format](./src-tauri/schema/app.capnp) using [Cap'n Proto](https://capnproto.org/) in your app data directory.

## Features

* Native desktop notification (Mac only right now, Windows + Linux Soon)
* [PLANNED] Customizable reminder interval
* [PLANNED] Beautiful statistics on how much water you drank
* [PLANNED] Google Fit integration
* [PLANNED] Apple Health integration
* [PLANNED] Less energy consumption

## Development - Prerequisites

You would need to follow the [Tauri](https://tauri.studio/) [installation guide](https://tauri.app/v1/guides/getting-started/prerequisites) to setup your development environment.

> *TL;DR: Setup MSVC Build Tools / XCode / build-essentials + webkit2gtk & Rust Nightly*

The app uses [Cap'n Proto](https://capnproto.org/install.html) to save / parse app data to binary. You should set it up for your platform and make it available on your `PATH`.  
You can simply run the following to install Cap'n Proto on your platform:
* Windows: `winget install capnproto.capnproto` or `choco install capnproto` 
* MacOS: `brew install capnp`
* Linux: `sudo apt install capnp` / `payman -S capnproto` / ... check your package manager

## Development - Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
