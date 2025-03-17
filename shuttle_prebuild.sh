#!/usr/bin/env bash

apt update && apt install -y nodejs

echo "Enabling pnpm via corepack..."
corepack enable pnpm

echo "Installing dependencies (frozen lockfile)..."
pnpm install --frozen-lockfile

echo "Building the project..."
pnpm build

echo "Done!"
