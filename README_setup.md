Setup instructions
=====================

As noted in [README.md](./README.md), we only support compiling from sources.
> **NOTE:** These installation instructions are aimed for Debian GNU/Linux (Testing).
> It should work also for other Linux distros, and probably for BSD too.
> The software is not tested under Windows or MacOS. 
> It might work (no reason for it to not to work), but it's completely untested.

## Rust 
First, make sure you have Rust installed.

    $ rustup -V
    rustup 1.24.3 (ce5817a94 2021-05-31)
    info: This is the version for the rustup toolchain manager, not the rustc compiler.
    info: The currently active `rustc` version is `rustc 1.60.0-nightly (f624427f8 2022-02-06)`

    $ cargo -V
    cargo 1.60.0-nightly (25fcb13 2022-02-01)

If these commands work, you're set. If they don't, please refer to 
https://rustup.rs/ for installing.

> **NOTE:** Yarrosco doesn't require Rust to be installed via rustup/curl. As long as
cargo is working, it should suffice.

## Typescript
(this is only needed if you need to tweak javascript code)

Check if you have already Typescript in your system:

    $ tsc --version
    Version 4.6.3

If not, refer to https://www.typescriptlang.org/download

It's usually installed by executing `npm install -g typescript`, but for this 
you need to install node.

* Linux Distro Packages: https://nodejs.org/en/download/package-manager/
* Direct: https://nodejs.org/en/download/

The command `npm` should be included on the node.js installation.

## Clone this repository in Git

> **NOTE:** If you don't use Git, simply download the ZIP file for the sources as
> provided by Codeberg or GitHub and unzip it.

I usually put all my Rust repositories in `~/git/rust/`. If you want to do the
same:

    $ mkdir -p ~/git/rust
    $ cd ~/git/rust

Clone this repo:

    $ git clone https://codeberg.org/yarmo/yarrosco.git

> **NOTE:** For the remaining of this guide, it is assumed that Yarrosco lives in **~/git/rust/yarrosco**

## Build Yarrosco

On the main folder (~/git/rust/yarrosco) run `cargo build`:

    $ cargo build
      Compiling yarrtwitch v0.1.0 (/home/deavid/git/rust/yarrosco/yarrtwitch)
      Compiling yarrosco v0.1.0 (/home/deavid/git/rust/yarrosco/yarrosco)
        Finished dev [unoptimized + debuginfo] target(s) in 13.44s

The first time you build it can take several minutes, as it will download all
dependencies and build all of them. It can take a while.

After the first build is done, any new build should take less than one minute.

## Set up the config files

On this folder we have two template files:
* yarrosco.template.toml
* yarrsecrets.template.toml

Duplicate them removing the ".template" part of the name:

    $ cp yarrosco.template.toml yarrosco.toml
    $ cp yarrsecrets.template.toml yarrsecrets.toml

Please refer to the file [README_config.md](./README_config.md) for details on
how to configure Yarrosco.

## Running yarrosco

You can run Yarrosco with:

    $ cargo run --bin yarrosco

You can also build it and run the binary directly:

    $ cargo build
    $ ./target/debug/yarrosco

Or build it in release mode to get a faster executable:

    $ cargo build --release
    $ ./target/release/yarrosco

> **NOTE:** At this point, the Yarrosco server is already working, but you'll have
> no useful way to output the messages to OBS. Continue reading for the details.

> **WARNING:** Yarrosco defaults to read all configs and write all files into
> the current working folder. This means that you **MUST** launch Yarrosco from
> it's home folder. You can run it from any other folder, but you need to move
> the files there.

## Controlling logging

Yarrosco reads the environment variable `RUST_LOG` to control the amount of logs.

The full details can be seen in https://docs.rs/env_logger/latest/env_logger/

By default, logs are set to INFO. If you want more logs:

    $ RUST_LOG=debug cargo run --bin yarrosco

If you want less logs:

    $ RUST_LOG=warn cargo run --bin yarrosco

If you want even less logs:

    $ RUST_LOG=error cargo run --bin yarrosco

All logs are sent to stderr. You can also pipe them somewhere else:

    $ RUST_LOG=error cargo run --bin yarrosco 2>>~/yarrosco.log

## Yarrosco's message database

Currently the Yarrosco server just outputs to two files:

* `yarrdb_log.jsonl`: All incoming messages go here. From time to time this file
  is emptied and the contents dumped into `yarrdb_data.jsonl`
* `yarrdb_data.jsonl` is the checkpointing database file. Every program start 
  and every N (usually 100) messages the log file gets integrated here. This 
  file only keeps the last N messages in chronological order.

The format for these files is JSONL or JSON-lines. It's basically one JSON per 
line.

These files are automatically created at start-up if they don't exist. If they
do exist, they are parsed and they initialize the database of messages so 
Yarrosco will remember the last chats received.

> **NOTE:** Some chat providers like Twitch don't provide history of messages,
> so the chats sent while Yarrosco is not running will be lost.

> **NOTE:** Some web servers and other programs rely on metadata to detect 
> changes in a file. Yarrosco will keep the log file open constantly
> until it does a checkpoint. This can confuse some web servers into thinking
> the file hasn't changed.

> **NOTE:** Currently Yarrosco doesn't offer any endpoints to communicate with.
> Your only option at the moment is directly reading the database files.

## Displaying messages in OBS

For convenience we provided with a small Typescript+HTML application that can
read Yarrosco's database and format the chats. At the current moment we don't 
have any configuration options for this, and it's highly likely you need to
adapt it for your own needs. Having Typescript installed is mandatory for these
purposes.

Please refer to [data/README_setup.md](./data/README_setup.md) for a guide on
how to set it up so you can get the messages displayed in a browser.

If you want to customize it, have a look to [data/README_customize.md](./data/README_customize.md)





