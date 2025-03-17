#!/usr/bin/env bash

apt-get update && apt-get install -y nodejs npm

# Now install pnpm globally
npm install -g pnpm

echo "Installing dependencies..."
pnpm install

echo "Building..."
pnpm run build