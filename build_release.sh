mkdir "release/$1"
cargo build --release
cargo build --release --target x86_64-pc-windows-gnu
cp target/release/bilibili-live-danmaku-cli "release/$1/bilibili-live-danmaku-cli-$1.x86_64_linux"
cp target/x86_64-pc-windows-gnu/release/bilibili-live-danmaku-cli.exe "release/$1/bilibili-live-danmaku-cli-$1.x86_64_windows-gnu.exe"
