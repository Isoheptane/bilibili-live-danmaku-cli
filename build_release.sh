mkdir "release/$1"
cargo build --release --target x86_64-unknown-linux-musl
cargo build --release --target x86_64-pc-windows-gnu

cp target/x86_64-unknown-linux-musl/release/bilibili-live-danmaku-cli "release/$1/bilibili-live-danmaku-cli-$1.x86_64_linux-musl"
cp target/x86_64-pc-windows-gnu/release/bilibili-live-danmaku-cli.exe "release/$1/bilibili-live-danmaku-cli-$1.x86_64_windows-gnu.exe"
