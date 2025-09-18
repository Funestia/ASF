#!/bin/sh
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target x86_64-pc-windows-gnu
cp ./target/x86_64-pc-windows-gnu/release/asf.exe .
cp ./target/x86_64-unknown-linux-gnu/release/asf .
strip asf.exe
strip asf
tar -czf ./target/asf.tar.gz ./arms.txt ./body.txt ./head.txt ./legs.txt ./decorations.txt ./skills.txt ./waist.txt ./components.txt ./asf ./asf.exe ./Languages
zip -FSr ./target/asf.zip ./arms.txt ./body.txt ./head.txt ./legs.txt ./decorations.txt ./skills.txt ./waist.txt ./components.txt ./asf ./asf.exe ./Languages
rm asf asf.exe
