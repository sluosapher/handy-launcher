# Build and Packaging Guide
## Handy Launcher

**Version:** 1.0  
**Date:** March 16, 2026  
**Status:** Draft

---

## 1. Overview

### 1.1 Build Targets

| Platform | Target Triple | Output Format |
|----------|---------------|---------------|
| Windows x64 | `x86_64-pc-windows-msvc` | `.msi`, `.exe` |
| macOS Intel | `x86_64-apple-darwin` | `.dmg`, `.app` |
| macOS Apple Silicon | `aarch64-apple-darwin` | `.dmg`, `.app` |

### 1.2 CI/CD Pipeline

```
Development
    |
    v
Local Build & Test
    |
    v
Git Push
    |
    v
GitHub Actions
    |-- Build (Windows, macOS Intel, macOS ARM)
    |-- Test (Unit, Integration)
    |-- Package (.msi, .dmg)
    |-- Sign (Windows: signtool, macOS: codesign + notarize)
    |-- Release Draft
    |
    v
Manual QA
    |
    v
Release Published
```

---

## 2. Local Development Builds

### 2.1 Debug Build

```bash
# Fast build, unoptimized, includes debug symbols
cargo tauri build --debug

# Output:
# - src-tauri/target/debug/handy-launcher.exe (or .app)
# - ~50-100 MB (larger due to debug info)
```

### 2.2 Release Build

```bash
# Optimized build for distribution
cargo tauri build --release

# Output:
# - src-tauri/target/release/handy-launcher
# - ~5-10 MB (smaller, optimized)
```

### 2.3 Build Outputs

After building, find outputs at:

```
src-tauri/target/
u251cu2500u2500 debug/
u2502   u251cu2500u2500 handy-launcher      # Executable (unoptimized)
u2502   u2514u2500u2500 ...
u251cu2500u2500 release/
u2502   u251cu2500u2500 handy-launcher      # Executable (optimized)
u2502   u251cu2500u2500 bundle/
u2502       u251cu2500u2500 msi/              # Windows installer
u2502       u2502   u2514u2500u2500 *.msi
u2502       u251cu2500u2500 dmg/              # macOS disk image
u2502           u2514u2500u2500 *.dmg
u2514u2500u2500 ...
```

---

## 3. Platform-Specific Builds

### 3.1 Windows

**Prerequisites:**
- Visual Studio 2022 Build Tools
- Windows SDK 10.0.22000.0+
- WiX Toolset v3 (for MSI creation)

**Build:**
```powershell
# Native build on Windows
cargo tauri build --target x86_64-pc-windows-msvc

# Or explicit bundle command
cargo tauri bundle --bundles msi
```

**Output:**
```
src-tauri/target/release/bundle/msi/
+-- handy-launcher_1.0.0_x64.msi      # ~3-5 MB
    
src-tauri/target/release/
+-- handy-launcher.exe                # ~5-8 MB
```

**Installer Features:**
- Per-machine or per-user install
- Start menu shortcut
- Auto-detect previous version (upgrade flow)
- Clean uninstall (removes app, keeps user data)

### 3.2 macOS

**Prerequisites:**
- macOS 12+ (Monterey)
- Xcode 14+
- Apple Developer account (for signing/notarization)

**Build Universal Binary (Intel + Apple Silicon):**

```bash
# Build both targets
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

# Create universal binary
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# Combine with lipo
mkdir -p target/universal/release
lipo -create \
  target/x86_64-apple-darwin/release/handy-launcher \
  target/aarch64-apple-darwin/release/handy-launcher \
  -output target/universal/release/handy-launcher

# Package
cargo tauri bundle --bundles dmg
```

**Output:**
```
src-tauri/target/release/bundle/dmg/
+-- handy-launcher_1.0.0.dmg          # ~5-8 MB

src-tauri/target/universal/release/
+-- handy-launcher                    # Universal binary
```

**Bundle Structure:**
```
handy-launcher.app/
+-- Contents/
    +-- Info.plist
    +-- MacOS/
    ¦   +-- handy-launcher            # Universal binary
    +-- Resources/
        +-- icon.icns
```

---

## 4. Cross-Compilation

### 4.1 From macOS (Build for Windows)

```bash
# Install Windows target
rustup target add x86_64-pc-windows-msvc

# Requires cross-compilation toolchain
# Option A: Use cargo-xwin (simpler)
cargo install cargo-xwin
cargo xwin build --target x86_64-pc-windows-msvc

# Option B: Use cross (Docker-based)
cargo install cross
cross build --release --target x86_64-pc-windows-gnu
```

**Note:** Windows MSI creation requires Windows. Use CI/CD for Windows builds.

### 4.2 From Windows (Build for macOS)

Not supported natively. Use CI/CD (GitHub Actions) for macOS builds.

### 4.3 Recommended Approach

**Use GitHub Actions for all cross-platform builds:**
- Local dev: Build for current platform only
- Releases: Let CI build for all platforms simultaneously

---

## 5. Code Signing

### 5.1 Windows Signing

**Using signtool (Authenticode):**

```powershell
# Sign executable
signtool sign /f certificate.pfx /p password `
  /tr http://timestamp.digicert.com /td sha256 `
  /fd sha256 `
  target\release\handy-launcher.exe

# Sign MSI
signtool sign /f certificate.pfx /p password `
  /tr http://timestamp.digicert.com /td sha256 `
  /fd sha256 `
  target\release\bundle\msi\*.msi
```

**Using Azure Key HSM (recommended orgs):**

```powershell
# With Azure SignTool
azuresigntool sign `
  --key-vault-url https://my-vault.vault.azure.net `
  --certificate-name my-cert `
  --timestamp-rfc3161 http://timestamp.digicert.com `
  target\release\bundle\msi\*.msi
```

**Certificate Requirements:**
- Code Signing Certificate (OV or EV)
- EV recommended: shows "Verified Publisher" in SmartScreen
- Cost: ~$200-700/year (OV), ~$700-1500/year (EV)

### 5.2 macOS Signing & Notarization

**Sign App Bundle:**

```bash
# Sign with Developer ID
codesign --force --options runtime \
  --sign "Developer ID Application: Your Name (TEAM_ID)" \
  --entitlements src-tauri/entitlements.plist \
  --deep --verify \
  target/release/bundle/macos/handy-launcher.app

# Verify signature
codesign --verify --verbose \
  target/release/bundle/macos/handy-launcher.app
```

**Notarize (required for macOS 10.15+):**

```bash
# Create zip for notarization
ditto -c -k --keepParent \
  target/release/bundle/macos/handy-launcher.app \
  handy-launcher.zip

# Submit for notarization
# Option A: Using notarytool (modern)
xcrun notarytool submit handy-launcher.zip \
  --apple-id "your@email.com" \
  --team-id "TEAM_ID" \
  --password "@keychain:NOTARY_PASSWORD" \
  --wait

# Staple ticket (optional but recommended)
xcrun stapler staple \
  target/release/bundle/macos/handy-launcher.app
```

**Provisioning Profile (optional):**
- Only needed for specialized entitlements
- Most apps work with default entitlements

### 5.3 Signing Configuration in Tauri

**tauri.conf.json:**

```json
{
  "tauri": {
    "bundle": {
      "windows": {
        "certificateThumbprint": null,
        "digestAlgorithm": "sha256",
        "timestampUrl": "http://timestamp.digicert.com"
      },
      "macOS": {
        "entitlements": "entitlements.plist",
        "signingIdentity": null
      }
    }
  }
}
```

**entitlements.plist:**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <!-- Tauri requires these -->
    <key>com.apple.security.cs.allow-jit</key>
    <true/>
    <key>com.apple.security.cs.allow-unsigned-executable-memory</key>
    <true/>
    
    <!-- Network access for Ollama API -->
    <key>com.apple.security.network.client</key>
    <true/>
    <key>com.apple.security.network.server</key>
    <true/>
    
    <!-- File system access -->
    <key>com.apple.security.files.user-selected.read-write</key>
    <true/>
</dict>
</plist>
```

---

## 6. Automated Builds (CI/CD)

### 6.1 GitHub Actions Workflow

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - "v*"

jobs:
  build-windows:
    runs-on: windows-2022
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Rust
        uses: dtolnay/rust-action@stable
        
      - name: Setup Bun
        uses: oven-sh/setup-bun@v1
        
      - name: Install dependencies
        run: |
          cd frontend
          bun install
          
      - name: Build
        run: cargo tauri build
        
      - name: Sign (Windows)
        if: env.CERTIFICATE != '''
        run: |
          echo "${{ secrets.CERTIFICATE }}" | base64 -d > cert.pfx
          signtool sign /f cert.pfx /p "${{ secrets.CERTIFICATE_PASSWORD }}" `
            /tr http://timestamp.digicert.com /td sha256 `
            /fd sha256 `
            src-tauri/target/release/bundle/msi/*.msi
            
      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: windows-installer
          path: src-tauri/target/release/bundle/msi/*.msi

  build-macos:
    runs-on: macos-14  # ARM runner
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Rust
        uses: dtolnay/rust-action@stable
        with:
          targets: x86_64-apple-darwin,aarch64-apple-darwin
          
      - name: Setup Bun
        uses: oven-sh/setup-bun@v1
        
      - name: Build Universal Binary
        run: |
          cd frontend
          bun install
          cd ..
          cargo build --release --target x86_64-apple-darwin
          cargo build --release --target aarch64-apple-darwin
          lipo -create \
            target/x86_64-apple-darwin/release/handy-launcher \
            target/aarch64-apple-darwin/release/handy-launcher \
            -output target/universal/release/handy-launcher
          cargo tauri bundle --bundles dmg
          
      - name: Sign & Notarize (macOS)
        if: env.APPLE_CERTIFICATE != '''
        run: |
          # Decode certificates
          echo "${{ secrets.APPLE_CERTIFICATE }}" | base64 -d > certificate.p12
          echo "${{ secrets.APPLE_API_KEY }}" | base64 -d > api_key.p8
          
          # Create keychain
          security create-keychain -p "${{ secrets.KEYCHAIN_PASSWORD }}" build.keychain
          security import certificate.p12 -k build.keychain -P "${{ secrets.APPLE_CERTIFICATE_PASSWORD }}" -T /usr/bin/codesign
          security set-keychain-settings -lut 21600 build.keychain
          security unlock-keychain -p "${{ secrets.KEYCHAIN_PASSWORD }}" build.keychain
          
          # Sign
          codesign --force --options runtime \
            --sign "Developer ID Application" \
            target/release/bundle/macos/handy-launcher.app
            
          # Notarize
          ditto -c -k --keepParent target/release/bundle/macos/handy-launcher.app app.zip
          xcrun notarytool submit app.zip \
            --apple-id "${{ secrets.APPLE_ID }}" \
            --team-id "${{ secrets.TEAM_ID }}" \
            --password "${{ secrets.APPLE_APP_PASSWORD }}" \
            --wait
            
      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: macos-installer
          path: src-tauri/target/release/bundle/dmg/*.dmg

  create-release:
    needs: [build-windows, build-macos]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Download artifacts
        uses: actions/download-artifact@v4
        
      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          draft: true
          files: |
            **/*.msi
            **/*.dmg
          body: |
            ## Changes
            See [CHANGELOG.md](../CHANGELOG.md)
```

### 6.2 Local Release Testing

Before tagging a release, test locally:

```bash
# Create test tag
git tag -a v0.0.0-test -m "Test build"

# Push (CI will run)
git push origin v0.0.0-test

# Delete after testing
git tag -d v0.0.0-test
git push --delete origin v0.0.0-test
```

---

## 7. Distribution

### 7.1 Website Distribution

**Hosting Options:**
- GitHub Releases (free, integrated with CI)
- Website CDN (Cloudflare, AWS S3)
- Auto-updater support (tauri-plugin-updater)

**Download Page Structure:**
```
/download
    ?platform=windows ?redirect to .msi
    ?platform=mac     ?detect architecture, redirect to .dmg
    ?platform=mac-intel ?redirect to x64 .dmg
    ?platform=mac-arm   ?redirect to ARM64 .dmg
```

### 7.2 Auto-Updater (Optional)

**Setup with tauri-plugin-updater:**

```toml
# Cargo.toml
[dependencies]
tauri-plugin-updater = "1.0"
```

```rust
// main.rs
use tauri_plugin_updater::UpdaterExt;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Update Server:**
- JSON endpoint with version info
- Tauri checks on startup
- Silent download, prompt to install

---

## 8. Version Management

### 8.1 Semantic Versioning

Format: `MAJOR.MINOR.PATCH`

- **MAJOR**: Breaking changes (new config format, API changes)
- **MINOR**: New features (new model profiles, UI improvements)
- **PATCH**: Bug fixes (download fixes, error handling)

### 8.2 Version Bump Checklist

```bash
# 1. Update version in all places
vim src-tauri/Cargo.toml        # version = "1.0.1"
vim src-tauri/tauri.conf.json   # version field
vim frontend/package.json       # version field
vim CHANGELOG.md                # Add release notes

# 2. Commit
git add -A
git commit -m "Bump version to 1.0.1"

# 3. Tag
git tag -a v1.0.1 -m "Release version 1.0.1"

# 4. Push
git push origin main
git push origin v1.0.1
```

### 8.3 Version Display in App

```typescript
// frontend/src/stores/version.ts
import { getVersion } from "@tauri-apps/api/app";
import { getTauriVersion } from "@tauri-apps/api/app";

export const appVersion = await getVersion();
export const tauriVersion = await getTauriVersion();
```

```svelte
<!-- About dialog -->
<AboutDialog>
  <p>Handy Launcher v{appVersion}</p>
  <p>Built with Tauri v{tauriVersion}</p>
</AboutDialog>
```

---

## 9. Troubleshooting Builds

### 9.1 Common Errors

**Error: `error running bundle_dmg` (macOS)**
```bash
# Solution: Install create-dmg
brew install create-dmg

# Or use appdmg
npm install -g appdmg
```

**Error: `light.exe not found` (Windows MSI)**
```bash
# Solution: Install WiX Toolset
# Download from: https://wixtoolset.org/
# Add to PATH: C:\Program Files (x86)\WiX Toolset v3.11\bin
```

**Error: `bundle identifier has app prefix` (macOS)**
```bash
# Solution: Change bundle identifier in tauri.conf.json
# From: "com.tauri.dev"
# To: "com.yourcompany.handylauncher"
```

### 9.2 Build Sizes

If binary is too large:

```toml
# Cargo.toml - Enable optimizations
[profile.release]
opt-level = 3      # Maximum optimization
lto = true         # Link-time optimization
strip = true       # Strip symbols
panic = "abort"    # Smaller panic handling
codegen-units = 1  # Better optimization (slower build)
```

**Expected sizes:**
- Windows `.exe`: 5-8 MB
- Windows `.msi`: 3-5 MB (compressed)
- macOS `.app`: 8-12 MB
- macOS `.dmg`: 5-8 MB (compressed)

### 9.3 Build Performance

**Speed up builds:**

```bash
# Use sccache for C++ compilation caching
cargo install sccache
export RUSTC_WRAPPER=sccache

# Use mold linker (Linux)
# Use lld linker (cross-platform, via Rustflags)

# Parallel builds (already default with cargo)
cargo build --release -j$(nproc)
```

---

## 10. Release Checklist

Before publishing:

- [ ] Version bumped in all files
- [ ] CHANGELOG.md updated
- [ ] All tests passing (`cargo test`)
- [ ] Manual QA on all platforms
- [ ] Signed (Windows: signtool, macOS: codesign + notarize)
- [ ] Git tag created (`v1.0.0`)
- [ ] CI builds successful
- [ ] Release notes written
- [ ] Downloads tested (fresh VM/install)
- [ ] Auto-updater tested (if enabled)

---

## 11. References

- [Tauri Distribution Guide](https://tauri.app/v1/guides/distribution/)
- [Tauri Code Signing](https://tauri.app/v1/guides/distribution/sign/)
- [Apple Code Signing](https://developer.apple.com/developer-id/)
- [Microsoft Code Signing](https://docs.microsoft.com/en-us/windows-hardware/drivers/dashboard/code-signing-cert-manage)
- [WiX Toolset Documentation](https://wixtoolset.org/documentation/manual/)

---

*Document created: March 16, 2026*
