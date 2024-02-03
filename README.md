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



## Development - Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
