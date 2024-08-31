if [ $1 == "run" ]
then
    exec ./target/release/rust-user-email-service
elif [ $1 == "build" ]
then
    exec cargo build --release
elif [ $1 == "dev" ]
then
    exec cargo run
elif [ $1 == "watch" ]
then
	export RUST_LOG=info
    exec cargo watch -x run
elif [ $1 == "i" ]
then
    exec cargo install --path .
elif [ $1 == "full_doc" ]
then
    exec cargo doc --workspace --no-deps --target-dir=docs/
elif [ $1 == "doc" ]
then
    exec rustdoc $2 --edition 2021 --output=./docs
else
    echo "ERROR: Comand not bound"
fi
