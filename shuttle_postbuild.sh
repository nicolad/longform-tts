#!/usr/bin/env bash

set -e

echo "Enabling pnpm via corepack..."
corepack enable pnpm

echo "Installing dependencies (frozen lockfile)..."
pnpm install --frozen-lockfile

echo "Building the project..."
pnpm build

echo "Done!"
