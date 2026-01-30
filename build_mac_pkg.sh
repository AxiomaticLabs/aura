#!/bin/bash
# macOS PKG Builder for AuraDB
# This script creates a .pkg installer for macOS

set -e

echo "🔨 Building AuraDB for macOS..."

# 1. Build Binaries
echo "Building release binaries..."
cargo build --release --bin aura-server
cargo build --release --bin aura-cli

# 2. Create Staging Directory Structure
echo "Creating package structure..."
PKG_ROOT="target/pkg"
rm -rf "$PKG_ROOT"
mkdir -p "$PKG_ROOT/usr/local/bin"
mkdir -p "$PKG_ROOT/Library/LaunchDaemons"
mkdir -p "$PKG_ROOT/var/lib/aura"
mkdir -p "$PKG_ROOT/var/log/aura"

# 3. Copy Files
echo "Copying binaries and configuration..."
cp "target/release/aura-server" "$PKG_ROOT/usr/local/bin/"
cp "target/release/aura" "$PKG_ROOT/usr/local/bin/"
cp "distribution/launchd/com.aura.db.plist" "$PKG_ROOT/Library/LaunchDaemons/"

# 4. Set Permissions
chmod 755 "$PKG_ROOT/usr/local/bin/aura-server"
chmod 755 "$PKG_ROOT/usr/local/bin/aura"
chmod 644 "$PKG_ROOT/Library/LaunchDaemons/com.aura.db.plist"

# 5. Create preinstall script
cat > "$PKG_ROOT/preinstall" << 'EOF'
#!/bin/bash
# Preinstall script for AuraDB on macOS

# Create _aura user if it doesn't exist
if ! dscl . -read /Users/_aura >/dev/null 2>&1; then
    echo "Creating _aura system user..."
    # Create system user with UID in system range
    MAX_ID=$(dscl . -list /Users UniqueID | awk 'BEGIN{max=0} {if($2>max) max=$2} END{print max+1}')
    if [ $MAX_ID -lt 500 ]; then MAX_ID=500; fi

    dscl . -create /Users/_aura
    dscl . -create /Users/_aura UserShell /usr/bin/false
    dscl . -create /Users/_aura RealName "AuraDB System User"
    dscl . -create /Users/_aura UniqueID $MAX_ID
    dscl . -create /Users/_aura PrimaryGroupID 20
    dscl . -create /Users/_aura NFSHomeDirectory /var/lib/aura
else
    echo "_aura user already exists"
fi

# Create directories
mkdir -p /var/lib/aura
mkdir -p /var/log/aura
chown _aura:admin /var/lib/aura
chown _aura:admin /var/log/aura
chmod 700 /var/lib/aura
chmod 755 /var/log/aura

echo "Preinstall completed"
EOF
chmod 755 "$PKG_ROOT/preinstall"

# 6. Create postinstall script
cat > "$PKG_ROOT/postinstall" << 'EOF'
#!/bin/bash
# Postinstall script for AuraDB on macOS

echo "Starting AuraDB service..."

# Load the launchd service
launchctl load /Library/LaunchDaemons/com.aura.db.plist

# Wait a moment for service to start
sleep 2

# Check if service is running
if launchctl list | grep -q com.aura.db; then
    echo "✅ AuraDB service started successfully"
else
    echo "❌ Failed to start AuraDB service"
    exit 1
fi

echo ""
echo "AuraDB installation completed!"
echo "Service: com.aura.db"
echo "Binaries: /usr/local/bin/aura-server, /usr/local/bin/aura"
echo ""
echo "To check status: sudo launchctl list com.aura.db"
echo "To view logs: tail -f /var/log/aura.log"
echo "To stop service: sudo launchctl unload /Library/LaunchDaemons/com.aura.db.plist"
echo "To start service: sudo launchctl load /Library/LaunchDaemons/com.aura.db.plist"
EOF
chmod 755 "$PKG_ROOT/postinstall"

# 7. Build PKG
echo "Building .pkg installer..."
pkgbuild --root "$PKG_ROOT" \
         --identifier com.aura.db \
         --version 0.1.0 \
         --scripts "$PKG_ROOT" \
         --install-location / \
         "target/aura-db-0.1.0.pkg"

echo "✅ macOS Installer built: target/aura-db-0.1.0.pkg"
echo ""
echo "To install: sudo installer -pkg target/aura-db-0.1.0.pkg -target /"