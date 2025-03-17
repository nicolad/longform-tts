#!/usr/bin/env bash

apt update && apt install -y nodejs

echo "Installing pnpm..."
wget -qO- https://get.pnpm.io/install.sh | sh -

echo "Installing dependencies (frozen lockfile)..."
pnpm install --frozen-lockfile

echo "Building the project..."
pnpm build

echo "Done!"
