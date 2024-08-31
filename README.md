# Rust Mixer TON API

Microservice that acts with TON Blockchain and Mixer contract.

### Before start

1. Install Rust
2. Install all deps
```sh
cargo install --path .
```
3. Run project in watch mode with
```sh
cargo watch -x run
```
or in simple development mode
```sh
cargo run dev
```

### Build documentation
If you need to make docs for whole project - run
```sh
cargo doc --no-deps
```
or you can create docs for specific file
```sh
rustdoc src/main.rs
```
all builded files will appear in target/docs/rust-email-user-service
but you can use sc.sh
!! Advice to use commands from sc.sh if something not work !!

### Helper CLI Util (sc.sh)
- `./sc.sh build` - builds production version of app
- `./sc.sh run` - runs production version of app
- `./sc.sh dev` - runs app in development mode
- `./sc.sh watch` - runs app in development mode but with watching for file changes
- `./sc.sh i` - installs all deps
- `./sc.sh full_doc` - builds docs for all project
- `./sc.sh doc [file_path]` - builds doc for one file by path

#### Author
Eugene K