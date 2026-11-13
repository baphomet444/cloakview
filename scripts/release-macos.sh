#!/usr/bin/env bash
# Builds, signs, and notarizes the macOS CloakView bundle, then notarizes and
# staples the .dmg itself (Tauri only notarizes the .app inside it).
# Requires .env.notary at the repo root (gitignored) defining:
#   APPLE_ID, APPLE_PASSWORD (an app-specific password), APPLE_TEAM_ID
set -euo pipefail
cd "$(dirname "$0")/.."

if [ ! -f .env.notary ]; then
  echo "Missing .env.notary — create it with APPLE_ID, APPLE_PASSWORD, APPLE_TEAM_ID" >&2
  exit 1
fi
set -a
# shellcheck disable=SC1091
source .env.notary
set +a

export PATH="$HOME/.cargo/bin:$PATH"
npm run tauri build

version=$(node -p "require('./src-tauri/tauri.conf.json').version")
dmg="src-tauri/target/release/bundle/dmg/CloakView_${version}_aarch64.dmg"

xcrun notarytool submit "$dmg" \
  --apple-id "$APPLE_ID" --password "$APPLE_PASSWORD" --team-id "$APPLE_TEAM_ID" \
  --wait
xcrun stapler staple "$dmg"
spctl -a -vv -t open --context context:primary-signature "$dmg"

# Version-free copy so .../releases/latest/download/CloakView-Pro_aarch64.dmg
# (the link given to Pro buyers) always resolves to the newest release.
pro_dmg="src-tauri/target/release/bundle/dmg/CloakView-Pro_aarch64.dmg"
cp "$dmg" "$pro_dmg"

shasum -a 256 "$dmg"
echo "Upload both to the GitHub release:"
echo "  gh release upload v${version} \"$dmg\" \"$pro_dmg\" --repo baphomet444/cloakview --clobber"
