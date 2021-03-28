echo "checks that this builds on std+no_std + that all tests run"
cargo build --all-targets
# cargo build --all-targets --release
cargo test --all-targets
# cargo test --all-targets --release
rustup target add thumbv7em-none-eabihf
cargo check --target thumbv7em-none-eabihf --no-default-features --features "microfft-complex"
cargo check --target thumbv7em-none-eabihf --no-default-features --features "microfft-real"


# run examples
cargo run --release --example mp3-samples
