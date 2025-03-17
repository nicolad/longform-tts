#!/usr/bin/env bash

apt update && apt install -y nodejs

echo "Installing pnpm..."
wget -qO- https://get.pnpm.io/install.sh | ENV="$HOME/.shrc" SHELL="$(which sh)" sh -

echo "Installing dependencies..."
pnpm install

echo "Building..."
pnpm run build