#!/bin/bash
# Usage: ./scripts/release.sh 0.1.0

VERSION=$1

if [ -z "$VERSION" ]; then
  echo "Usage: ./scripts/release.sh <version>"
  exit 1
fi

echo "Preparing Release v$VERSION..."
sed -i.bak "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
rm Cargo.toml.bak
git add Cargo.toml
git commit -m "chore: Bump version to $VERSION"
git tag -a "v$VERSION" -m "Release v$VERSION"
echo "Pushing to GitHub..."
git push origin main
git push origin "v$VERSION"
echo "Done! GitHub Action should be building release v$VERSION now."