./examples/wasm/web_build.sh ${@:1} ${@:2}
cargo install devserver
devserver --header Cross-Origin-Opener-Policy='same-origin' --header Cross-Origin-Embedder-Policy='require-corp' --path examples/wasm