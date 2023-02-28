set -e
set -x

echo "checks that this builds on std+no_std + that all tests run + that all features compile"
cargo build --all-targets
cargo build --all-targets --no-default-features --features "rustfft-complex"
cargo build --all-targets --no-default-features --features "microfft-real"

cargo test --all-targets
cargo test --all-targets --no-default-features --features "rustfft-complex"
cargo test --all-targets --no-default-features --features "microfft-real"

cargo bench

cargo fmt -- --check # (--check doesn't change the files)

cargo doc
cargo doc --no-default-features --features "rustfft-complex"
cargo doc --no-default-features --features "microfft-real"

cargo clippy --all-targets
cargo clippy --all-targets --no-default-features --features "rustfft-complex"
cargo clippy --all-targets --no-default-features --features "microfft-real"

# test no_std
rustup target add thumbv7em-none-eabihf
# nope, thats BS: this crate needs STD
# cargo check --target thumbv7em-none-eabihf --no-default-features --features "rustfft-complex"
cargo check --target thumbv7em-none-eabihf --no-default-features --features "microfft-real"

# run examples
cargo run --release --example mp3-samples
cargo run --release --example mp3-samples --no-default-features --features "rustfft-complex"
cargo run --release --example mp3-samples --no-default-features --features "microfft-real"
