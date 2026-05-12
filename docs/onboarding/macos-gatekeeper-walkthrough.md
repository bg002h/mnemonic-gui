# macOS Gatekeeper walkthrough — first-launch instructions

`mnemonic-gui` v0.1.0 binaries are NOT signed with an Apple Developer ID
(code-signing is deferred to v0.2 per FOLLOWUPS
`gui-secret-buffer-allocator-residue` and §B.14). On first launch, macOS
Gatekeeper will refuse to open the binary with a message like:

> "mnemonic-gui" cannot be opened because the developer cannot be verified.

This is expected. Two ways to proceed:

## Option A — Right-click → Open (recommended for casual users)

1. Open Finder, navigate to the extracted `mnemonic-gui` binary.
2. Right-click (or Control-click) `mnemonic-gui` → choose **Open**.
3. The system dialog now includes an **Open** button. Click it.
4. macOS remembers this exemption; subsequent launches open directly.

## Option B — Strip the quarantine attribute (command-line)

For users comfortable with the terminal, this removes the quarantine bit
once and avoids the Gatekeeper dialog entirely:

```bash
xattr -d com.apple.quarantine /path/to/mnemonic-gui
```

After this, double-clicking the binary launches it normally.

## Why this is necessary

Apple's notarization service requires a paid Developer ID ($99/yr) plus
a notarization roundtrip per release. v0.1.0 ships unsigned to keep the
release process simple while the project is in pre-release. The
mitigation is a v0.2 work item:

- FOLLOWUPS slug: `gui-code-signing-mac-developer-id`

## Verifying the binary

The release page on GitHub
(`https://github.com/bg002h/mnemonic-gui/releases/tag/mnemonic-gui-v0.1.0`)
publishes SHA-256 checksums for each artifact. After download, verify:

```bash
shasum -a 256 mnemonic-gui-v0.1.0-x86_64-macos.tar.gz
# Compare the output against the release page's published hash.
```

This is the v0.1 integrity check until code-signing lands.
