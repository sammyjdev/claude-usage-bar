#!/bin/bash
set -e
cd "$(dirname "$0")"

cargo build --release

APP="ClaudeUsageBar.app"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS"
cp target/release/claude-usage-bar "$APP/Contents/MacOS/claude-usage-bar"
cp macos/Info.plist "$APP/Contents/Info.plist"

echo "built: $(pwd)/$APP"
