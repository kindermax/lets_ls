# Lets task runner Language Server (lets-ls)

## Features

- **Go To Definition**: Navigate to definitions of `mixins`

## Configuration

Initialization options:

- **log_path**: location for LS log

## Installation

## Build from source

```sh
cargo build --release
```

Executable can then be found at _target/release/lets_ls

## Integration with VSCode

Extension can be found [here](https://marketplace.visualstudio.com/items?itemName=kindritskyimax.lets-ls).

This extension supports configuration which needs to be set up because _lets_ls_
itself isn't installed along with the extension but it needs to be downloaded from
releases, brew or built from source.