cargo +nightly build --target wasm32-unknown-unknown --example $1 ${@:2}
cp target/wasm32-unknown-unknown/debug/examples/$1.wasm examples/wasm/koi.wasm