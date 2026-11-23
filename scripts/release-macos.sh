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

# Publish to the private R2 bucket that /api/dl signs for. The S3 keys the site presigns with can
# also publish the artifact, so they live in ~/.cloakview-r2.env (mode 600) instead of the shell.
# Skipped when no credentials are reachable, so a release still works on an offline machine.
r2_env="${R2_ENV_FILE:-$HOME/.cloakview-r2.env}"
if [ -f "$r2_env" ] || { [ -n "${R2_ACCESS_KEY_ID:-}" ] && [ -n "${R2_ACCOUNT_ID:-}" ]; }; then
  echo "Uploading to R2 with the S3 credentials from ${r2_env}"
  python3 scripts/r2-put.py "$pro_dmg" "${R2_KEY:-CloakView-Pro_aarch64.dmg}"
elif [ -n "${CLOUDFLARE_API_TOKEN:-}" ] && [ -n "${R2_ACCOUNT_ID:-}" ]; then
  r2_key="${R2_KEY:-CloakView-Pro_aarch64.dmg}"
  r2_bucket="${R2_BUCKET:-cloakview-downloads}"
  echo "Uploading to R2 with wrangler: $r2_bucket/$r2_key"
  CLOUDFLARE_ACCOUNT_ID="$R2_ACCOUNT_ID" npx --yes wrangler@4 r2 object put "$r2_bucket/$r2_key" \
    --file "$pro_dmg" --content-type application/x-apple-diskimage --remote
else
  echo "R2 credentials not set -- skipping R2 upload (site falls back to GitHub)."
fi

echo "Upload both to the GitHub release:"
echo "  gh release upload v${version} \"$dmg\" \"$pro_dmg\" --repo baphomet444/cloakview --clobber"
