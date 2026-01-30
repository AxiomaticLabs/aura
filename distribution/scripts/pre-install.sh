#!/bin/bash
# Pre-install script for AuraDB
# Creates the 'aura' system user

set -e

# Create aura user if it doesn't exist
if ! id -u aura >/dev/null 2>&1; then
    useradd --system --shell /bin/false --home-dir /var/lib/aura --create-home aura
    echo "Created system user 'aura'"
else
    echo "User 'aura' already exists"
fi

# Create data directory
mkdir -p /var/lib/aura
chown aura:aura /var/lib/aura
chmod 700 /var/lib/aura

# Create log directory
mkdir -p /var/log/aura
chown aura:aura /var/log/aura
chmod 755 /var/log/aura

echo "Pre-install completed successfully"