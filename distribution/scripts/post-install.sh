#!/bin/bash
# Post-install script for AuraDB
# Enables and starts the systemd service

set -e

# Reload systemd daemon
systemctl daemon-reload

# Enable the service to start on boot
systemctl enable aura-server

# Start the service
systemctl start aura-server

echo "AuraDB service installed and started"
echo "Service status: $(systemctl is-active aura-server)"
echo ""
echo "To check logs: journalctl -u aura-server -f"
echo "To stop service: sudo systemctl stop aura-server"
echo "To restart service: sudo systemctl restart aura-server"