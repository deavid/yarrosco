Structure of the project, binaries and folders
==============================================

## Programs provided

* `yarrosco`: Main server/daemon program that will do mostly everything.
* `yarrpass`: Utility to create secrets for `yarrsecrets.toml` config.
* `yarrcfg`: Debugging utility to see how Yarrosco parses the config file.
* `yarrtwitch`: Sample program to test Twitch connection.
* `yarrmatrix`: Sample program to test Matrix connection.
* `yarrdata`: Playground to test message passing and database storage.

Additionally:
* `data` folder contains the HTML+TS application to show messages on OBS.

Binaries are usually built in:
* `target/debug/...` for regular `cargo build` and `cargo run` invocations.
* `target/release/...` for release builds, using `cargo build --release` and `cargo run --release`.

## Workspace Crates

* `yarrpass` defines how to cipher and decipher secrets for configs.
* `yarrcfg` is in charge of parsing the config files.
  * depends on `yarrpass` to correctly parse secrets in the config files.
* `yarrdata` manages the interface for receiving and sending chat messages
  * `db.rs` implements the database of JSONL files.
* `yarrtwitch` has the service for reading Twitch chat messages.
  * depends on `yarrcfg` to understand the configuration data.
  * depends on `yarrdata` to export the messages received.
* `yarrmatrix` has the service for reading Matrix chat messages.
  * depends on `yarrcfg` to understand the configuration data.
  * depends on `yarrdata` to export the messages received.
* `yarrosco` implements the full server that spawns `yarrtwitch` and `yarrmatrix`
  services as well as the `yarrdata::db` to read/write on disk.
  * depends on: `yarrcfg` `yarrdata` `yarrtwitch` `yarrmatrix`



