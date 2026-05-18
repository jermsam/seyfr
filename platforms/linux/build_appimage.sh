#!/bin/bash
# SEYFR Linux AppImage Build & Package Orchestrator
# This script bundles SEYFR (GTK4 + PyGObject + Rust Core) into a single, signed AppImage.

set -e

echo "📦 Preparing SEYFR Linux AppImage Build..."

# 1. Platform Check
OS_TYPE=$(uname -s)
if [ "$OS_TYPE" != "Linux" ]; then
    echo "⚠️  Warning: AppImage packaging requires a Linux host system with 'appimagetool' and library compilers."
    echo "   We are generating the packaging scripts and GitHub Actions pipeline so this runs automatically in the cloud on every release!"
fi

# 2. Setup directory paths
REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PLATFORM_DIR="${REPO_DIR}/platforms/linux"
BUILD_DIR="${REPO_DIR}/target/appimage"
APPDIR="${BUILD_DIR}/Seyfr.AppDir"

echo "🧹 Cleaning previous build artifacts..."
rm -rf "${BUILD_DIR}"
mkdir -p "${APPDIR}/usr/bin"
mkdir -p "${APPDIR}/usr/lib"
mkdir -p "${APPDIR}/usr/share/icons/hicolor/512x512/apps"

# 3. Compile Rust Core (only if on Linux)
if [ "$OS_TYPE" == "Linux" ]; then
    echo "🦀 Compiling Rust Core in Release mode..."
    cargo build --release --manifest-path "${REPO_DIR}/core/Cargo.toml"
    
    echo "🧬 Generating Python UniFFI Bindings..."
    if ! command -v uniffi-bindgen &> /dev/null; then
        echo "   Installing uniffi-bindgen..."
        cargo install uniffi --features cli --bin uniffi-bindgen --version 0.31.1
    fi
    
    # Generate bindings
    uniffi-bindgen generate \
        --library "${REPO_DIR}/target/release/libseyfr_core.so" \
        --language python \
        --out-dir "${PLATFORM_DIR}"
fi

# 4. Copy Python sources & assets to AppDir/usr/bin
echo "📂 Assembling AppDir structure..."
cp "${PLATFORM_DIR}/main.py" "${APPDIR}/usr/bin/"
cp "${PLATFORM_DIR}/app.py" "${APPDIR}/usr/bin/"
cp "${PLATFORM_DIR}/window.py" "${APPDIR}/usr/bin/"
cp "${PLATFORM_DIR}/core_wrapper.py" "${APPDIR}/usr/bin/"
cp "${PLATFORM_DIR}/style.css" "${APPDIR}/usr/bin/"

# Copy generated bindings if they exist
if [ -f "${PLATFORM_DIR}/seyfr_core.py" ]; then
    cp "${PLATFORM_DIR}/seyfr_core.py" "${APPDIR}/usr/bin/"
fi

# Copy compiled shared library if it exists
if [ -f "${REPO_DIR}/target/release/libseyfr_core.so" ]; then
    cp "${REPO_DIR}/target/release/libseyfr_core.so" "${APPDIR}/usr/bin/"
elif [ -f "${PLATFORM_DIR}/libseyfr_core.so" ]; then
    cp "${PLATFORM_DIR}/libseyfr_core.so" "${APPDIR}/usr/bin/"
fi

# 5. Copy Icons and Desktop Entries
echo "🎨 Adding brand icons & desktop configurations..."
cp "${REPO_DIR}/playstore-assets/app_icon.png" "${APPDIR}/usr/share/icons/hicolor/512x512/apps/seyfr.png"
cp "${REPO_DIR}/playstore-assets/app_icon.png" "${APPDIR}/seyfr.png"

# Write Desktop Entry file
cat > "${APPDIR}/seyfr.desktop" <<EOF
[Desktop Entry]
Name=Seyfr
Exec=seyfr %U
Icon=seyfr
Type=Application
Categories=Utility;FileTransfer;
Comment=Next-gen Zero-Cloud Peer-to-Peer File Transfer
Terminal=false
StartupWMClass=com.jitpomi.seyfr
EOF

# Write AppRun entrypoint script
cat > "${APPDIR}/AppRun" <<'EOF'
#!/bin/sh
# SEYFR self-contained entrypoint loader
HERE="$(dirname "$(readlink -f "${0}")")"

export PYTHONPATH="${HERE}/usr/bin:${PYTHONPATH}"
export LD_LIBRARY_PATH="${HERE}/usr/bin:${LD_LIBRARY_PATH}"

# Check for GTK4/Libadwaita dependencies
python3 -c "import gi; gi.require_version('Gtk', '4.0')" 2>/dev/null
if [ $? -ne 0 ]; then
    echo "⚠️  Error: SEYFR requires GTK4 and Libadwaita to run."
    echo "   Please run: sudo apt install python3-gi python3-gi-cairo gir1.2-gtk-4.0 gir1.2-adw-1"
    exit 1
fi

exec python3 "${HERE}/usr/bin/main.py" "$@"
EOF
chmod +x "${APPDIR}/AppRun"

# 6. Build AppImage (only if on Linux and appimagetool is available)
if [ "$OS_TYPE" == "Linux" ]; then
    echo "📦 Packaging AppImage using appimagetool..."
    if [ ! -f "${BUILD_DIR}/appimagetool" ]; then
        echo "   Downloading appimagetool..."
        wget -q -O "${BUILD_DIR}/appimagetool" "https://github.com/AppImage/AppImageKit/releases/download/13/appimagetool-x86_64.AppImage"
        chmod +x "${BUILD_DIR}/appimagetool"
    fi
    
    # Run appimagetool
    ARCH=x86_64 "${BUILD_DIR}/appimagetool" "${APPDIR}" "${BUILD_DIR}/Seyfr_amd64.AppImage"
    
    # 7. Crytographic GPG Signing
    if gpg --list-secret-keys "ssali@jitpomi.com" &>/dev/null; then
        echo "✍️  Cryptographically signing AppImage with GPG key..."
        gpg --armor --detach-sign "${BUILD_DIR}/Seyfr_amd64.AppImage"
        echo "✅ Signature file 'Seyfr_amd64.AppImage.asc' generated successfully!"
    else
        echo "ℹ️  No GPG secret key found for 'ssali@jitpomi.com'. Skipping local GPG signing step."
    fi
    
    echo "🎉 SEYFR Linux packaging complete!"
    echo "   Output: ${BUILD_DIR}/Seyfr_amd64.AppImage"
else
    echo "💡 Local packaging script generated at: platforms/linux/build_appimage.sh"
    echo "   Use this script on any Linux machine, or let GitHub Actions build it automatically on push!"
fi
