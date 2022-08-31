parent_path=$( cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd -P )
cd "$parent_path"

cargo +nightly build --target wasm32-unknown-unknown --example $1 ${@:2}
cp ../../target/wasm32-unknown-unknown/debug/examples/$1.wasm koi.wasm