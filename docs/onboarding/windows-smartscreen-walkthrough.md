# Windows SmartScreen walkthrough — first-launch instructions

`mnemonic-gui` v0.1.0 binaries are NOT signed with Authenticode
(code-signing is deferred to v0.2 per FOLLOWUPS `gui-code-signing-windows`
and §B.14). On first launch, Windows SmartScreen will refuse to open the
binary with a message like:

> Windows protected your PC
> Microsoft Defender SmartScreen prevented an unrecognized app from
> starting. Running this app might put your PC at risk.

This is expected. To proceed:

1. Click **More info** in the SmartScreen dialog.
2. A **Run anyway** button appears below the app name. Click it.
3. SmartScreen remembers this exemption; subsequent launches open directly.

## Why this is necessary

Microsoft Authenticode code-signing requires a paid certificate (~$200/yr
for the basic EV variant). v0.1.0 ships unsigned to keep the release
process simple while the project is in pre-release. The mitigation is a
v0.2 work item:

- FOLLOWUPS slug: `gui-code-signing-windows`

## Verifying the binary

The release page on GitHub
(`https://github.com/bg002h/mnemonic-gui/releases/tag/mnemonic-gui-v0.1.0`)
publishes SHA-256 checksums for each artifact. After download, verify in
PowerShell:

```powershell
Get-FileHash -Algorithm SHA256 .\mnemonic-gui-v0.1.0-x86_64-windows.zip
# Compare the output against the release page's published hash.
```

This is the v0.1 integrity check until code-signing lands.
