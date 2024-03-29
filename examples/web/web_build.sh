parent_path=$( cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd -P )
cd "$parent_path"
RUSTFLAGS='-C target-feature=+atomics,+bulk-memory,+mutable-globals -Clink-arg=--max-memory=4294967296' \
    cargo +nightly build --target wasm32-unknown-unknown -Z build-std=std,panic_abort --release --example $1 ${@:2}
cp ../../target/wasm32-unknown-unknown/release/examples/$1.wasm koi.wasm