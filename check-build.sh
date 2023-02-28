set -e
set -x

echo "checks that this builds on std+no_std + that all tests run + that all features compile"
cargo build --all-targets

cargo test --all-targets

cargo bench

cargo fmt -- --check # (--check doesn't change the files)

cargo doc --document-private-items

cargo clippy --all-targets

# test no_std
rustup target add thumbv7em-none-eabihf
cargo build --target thumbv7em-none-eabihf

# run examples
cargo run --release --example mp3-samples
