# Releasing PhotoCraft

> **This fork (685ks/photocraft) releases differently from upstream.** Run **Actions > Release >
> Run workflow** with the version (the `[workspace.package] version` in `Cargo.toml`, e.g.
> `0.11.1`). `.github/workflows/release.yml` calls the shared Arthavan Projects workflow
> `685ks/craft-actions/.github/workflows/release-rust-app.yml@main` (package `photocraft`, binary
> `photocraft`), which reads `packaging/app.toml`, builds Windows x64 (`.msi`, `-portable.zip`) and
> creates the release `v<version>` with `SHA256SUMS.txt`, an SBOM and build provenance (a draft
> unless `publish` is ticked). Assets are named `photocraft-<version>-<os>-<arch>[-portable].<ext>`.
> The Arthavan Projects launcher installs and updates PhotoCraft from these releases; the app has
> no updater of its own. The upstream process below (the `release` branch, FreeBSD, web and
> Windows ARM64 builds) is kept for reference and for the packaging scripts it describes. This
> fork ships for Windows only: the macOS, Linux and FreeBSD packaging was removed.

Every push to the `release` branch runs `.github/workflows/release.yml`. The workflow builds
signed installers for Windows and the web, then creates
or updates a **draft** GitHub Release named `PhotoCraft v<version>`. Nobody sees a draft until a
maintainer publishes it.

This is PhotoCraft's implementation of the shared
[release playbook](release-playbook.md). User-facing names say **PhotoCraft**. Files, binaries
and ids stay lowercase (`photocraft-<version>-<platform>-<arch>.<ext>`, `app.freedev.ainz.ks.PhotoCraft`).

## Cutting a release

1. **Bump the version** on `main`. The only place it lives is `[workspace.package] version`
   in the root `Cargo.toml`:

   ```sh
   cargo xtask version                 # prints the current version, e.g. 0.1.0
   cargo xtask version set 0.2.0       # or 0.2.0-rc.1; updates Cargo.toml and Cargo.lock
   ```

   Commit the change (`Cargo.toml` + `Cargo.lock`) through the normal review flow.
2. **Merge `main` into `release`** (or fast-forward it) and push. The workflow starts by itself.
3. **Wait for the draft.** After about 30 to 45 minutes (notarization is the slow part), the
   Releases page has a draft `PhotoCraft v0.2.0`, tagged `v0.2.0` on the pushed commit, with
   every artifact and `SHA256SUMS.txt`. The notes are generated from the merged PRs.
4. **Check it.** Download an installer or two and look at the job summaries. Any
   `::warning::` there means a signing secret was missing and that artifact is unsigned.
5. **Add the scorecard deltas** to the draft's notes: what moved in
   [`docs/scorecard.md`](scorecard.md) since the previous release. Compare the summary table
   and the performance table of both tags (`git diff v<previous> v<this> -- docs/scorecard.md`)
   and list each area's done / partial / missing change, the scenarios that newly meet their
   budget or got slower, corpus floors raised, and the change in settings that do nothing.
   Quote numbers with the baseline's machine and load average.
6. **Publish** the draft in the GitHub UI. Publishing creates the `v0.2.0` tag. Versions with a
   pre-release suffix (`-rc.1`) are marked as pre-releases.

Pushing to `release` again before you publish rebuilds the same draft and replaces its assets.
After the draft is published, the workflow refuses to touch that version again, so bump it first.

**Test runs:** *Actions → Release → Run workflow* runs the whole pipeline by hand. The optional
`version` input (such as `0.2.0-rc.1`) overrides `Cargo.toml` for that run only. The jobs apply
it with `cargo xtask version set` before building, so the binaries report it too. The run still
needs the `release` environment, which only the `release` branch can use, so pick that branch
in the dialog.

## What gets built

| Platform | Artifacts | Built on |
|---|---|---|
| Windows 10+ x64 | `photocraft-<v>-windows-x64.msi`, `photocraft-<v>-windows-x64-portable.zip` | `windows-latest` |
| Windows 10+ x86 (32-bit) | `photocraft-<v>-windows-x86.msi`, `photocraft-<v>-windows-x86-portable.zip` | `windows-latest` |
| Web | `photocraft-web-<v>.zip` (static site; see [`packaging/web/README.md`](../packaging/web/README.md)) | `ubuntu-latest` |

Every binary reports its version, the commit and the build date: `photocraft --version`,
`photocraft-cli --version`, and *Help › About PhotoCraft*. CI sets `PHOTOCRAFT_BUILD_SHA` and
`PHOTOCRAFT_BUILD_DATE`, and `crates/engine/src/build_info.rs` reads them at compile time. A plain
`cargo build` doesn't set them and reports `0.2.0 (dev build)`.

Every desktop build job also checks out [craft-fonts](https://github.com/storytold/craft-fonts)
at the `craft-fonts-ref` commit in `.github/workflows/release.yml` and builds with `CRAFT_FONTS_DIR` and
`CRAFT_FONTS_REQUIRED=1`, so desktop releases embed its Japanese fonts (the web build embeds none: see
`docs/development.md` › Fonts) and fail rather than ship without them. The packages carry each font's licence as
`OFL-<family>.txt` (`copy_font_licences` in `packaging/env.sh`; the portable zip on Windows;
`Contents/Resources/Licenses` in the macOS app). Bump the pin deliberately, together with the one
in `ci.yml`. Rules: `../craftrules/standards/fonts.md`; build option: `docs/development.md` › Fonts.

### Windows

`packaging/windows/package.ps1 -Arch x64|x86` builds with `-C target-feature=+crt-static`.
The static C runtime means neither the MSI nor the portable zip needs the Visual C++
redistributable, which matters for a standalone installer and costs only about 100 KB. The flag
goes in `CARGO_TARGET_<TRIPLE>_RUSTFLAGS`, so host build scripts aren't affected.

- `apps/photocraft/build.rs` and `apps/photocraft-cli/build.rs` embed the icon
  (`assets/app-icon/photocraft.ico`) and VERSIONINFO with the `winresource` crate
  (build dependency `0.1`). They only do this when targeting Windows. Elsewhere each
  script is a no-op, even if `PHOTOCRAFT_REQUIRE_WINRES` is set, and the web build
  doesn't touch that crate. Both set ProductName to PhotoCraft, CompanyName to
  Learning Machines LLC and the same copyright line. The GUI description is
  "PhotoCraft image editor" (`photocraft.exe`). The CLI description is
  "PhotoCraft command-line interface", with OriginalFilename `photocraft-cli.exe`
  and InternalName `photocraft-cli`. Version strings are the Cargo package version,
  so a pre-release tag is kept; the numeric file and product versions are
  winresource's `major.minor.patch.0`. A missing resource compiler warns, so an
  optional cross-compile still links. `package.ps1` sets `PHOTOCRAFT_REQUIRE_WINRES=1`.
  The CLI build script then returns an error naming `rc.exe` or `windres` and does
  not panic.
- Release builds use the GUI subsystem, so Start Menu launches don't open a console window.
  The CLI stays console subsystem 3 so its output reaches the terminal.
- `photocraft.wxs` (WiX v5) is a per-machine install into Program Files with an advertised
  Start Menu shortcut. PhotoCraft becomes the default app for `.pcraft` and is listed under
  "Open with" for PSD/PSB and image files. It also registers App Paths (Win+R `photocraft`).
  The MSI version is the numeric `X.Y.Z`, because MSI has no pre-release field. Same-version
  upgrades are allowed so that release candidates replace each other.
- Double-clicking the MSI opens a setup wizard (WixUI_InstallDir without the licence page):
  Welcome, install folder (remembered for upgrades in `HKLM\Software\PhotoCraft\InstallDir`),
  Ready, a progress page, and a Finish page that says where to find the app, with a ticked "Launch
  PhotoCraft" box. The pages name the full version (`photocraft.en-us.wxl`). `package.ps1` draws
  the wizard's banner and side bitmaps from the app icon, so no WiX stock art ships. `msiexec /qn`
  still installs silently. Building needs the WiX UI and Util extensions:
  `wix extension add -g WixToolset.UI.wixext/5.0.2 WixToolset.Util.wixext/5.0.2`.
- Shortcut icon identifiers keep the executable's `.exe` extension: MSI uses the identifier
  as the cached icon filename, and an extensionless filename can render as a blank document
  icon. `packaging/windows/check-icons.ps1` checks the references and extensions in CI;
  `package.ps1` also validates the built MSI with ICE50 before signing it.
- **Portable zip:** it ships `packaging/windows/portable.txt` beside `photocraft.exe`. That
  marker (or a `PhotoCraft.portable` file) switches on portable mode: preferences, presets,
  recovery autosaves and the GPU startup marker go to `PhotoCraftData\` next to the exe instead
  of `%APPDATA%\Photocraft`. If that folder isn't writable the app warns and uses `%APPDATA%`.
  The MSI has no marker. The logic is in `apps/photocraft/src/app_dirs.rs` and works the same
  on macOS and Linux.
- **Signing:** `packaging/windows/sign.ps1` signs both `.exe` files and then the `.msi` with
  `signtool`, using SHA-256 and an RFC 3161 timestamp. It uses whichever material is present:
  1. a `.pfx` file (`WINDOWS_CERTIFICATE`, `WINDOWS_CERTIFICATE_PASSWORD`), timestamped by
     DigiCert, or
  2. Azure Trusted Signing (`AZURE_*`): the script downloads the
     `Microsoft.Trusted.Signing.Client` dlib and timestamps with Microsoft's server.

  The script is the single place to change when the Windows signing setup changes.

Locally on Windows: `dotnet tool install -g wix --version 5.0.2`,
`wix extension add -g WixToolset.UI.wixext/5.0.2 WixToolset.Util.wixext/5.0.2`, then
`pwsh packaging/windows/package.ps1 -Arch x64`.

### Web

`packaging/web/package.sh` runs `trunk build --release` for the app (`apps/photocraft-web`, into
`dist/web`) and for the module that starts it (`apps/photocraft-web-boot`, into `dist/web-boot`).
It gzips the app's wasm, cuts it into parts of at most 20 MiB and zips the boot module's page with
the parts, sample `_headers` and `.htaccess` files and the hosting guide. The site only uses
relative URLs, so it works under any path and in an iframe. The wasm builds with the
size-optimized `wasm-release` Cargo profile (set in `apps/photocraft-web/index.html`), and the
script fails if any file in the site exceeds 24 MiB, below Cloudflare's 25 MiB per-file limit;
the app's wasm itself may be larger, since it ships as parts.
[`packaging/web/README.md`](../packaging/web/README.md) covers MIME types, compression,
caching, the iframe snippet and the `?webgl` / `?cpu` flags.

## Secrets

All secrets live in the repository's **`release` environment** (*Settings → Environments →
release*). Restrict its deployment branches to `release`. Every job in `release.yml` that
signs or publishes declares `environment: release`, so only pushes to that branch, or manual runs on it, can read the secrets. Each secret is optional. If one is missing, that platform's artifacts are unsigned and
the run shows a `::warning::`.

| Secret | Used for |
|---|---|
| `WINDOWS_CERTIFICATE` | base64 `.pfx` code-signing certificate |
| `WINDOWS_CERTIFICATE_PASSWORD` | password for that `.pfx` |
| `AZURE_TENANT_ID`, `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET` | service principal for Azure Trusted Signing (an alternative to the `.pfx`) |
| `AZURE_SIGNING_ENDPOINT` | e.g. `https://eus.codesigning.azure.net` |
| `AZURE_SIGNING_ACCOUNT`, `AZURE_CERT_PROFILE` | Trusted Signing account and certificate profile names |

`GITHUB_TOKEN` creates the release. Only the final job gets `contents: write`.

## Icons

`assets/app-icon/photocraft.svg` is the canonical icon: the upstream owner's drawing of a
nine-tailed kitsune, vectorised (MIT OR Apache-2.0, see `LICENSE.txt` there; palette and
geometry in `README.md`). `packaging/icons.sh` regenerates the 1024 px PNG, the `.icns`, the `.ico` (packed by
`cargo xtask ico`) and the hicolor PNGs from it. It needs `resvg`, plus `iconutil` on macOS.
The outputs are committed, so packaging never needs those tools.

## Checks

`.github/workflows/packaging-lint.yml` runs in seconds on any change to `packaging/`, the
workflows or the icons. It runs actionlint, shellcheck, a PowerShell parse, xmllint,
`desktop-file-validate`, `appstreamcli validate`, and a YAML check that the two Flatpak
manifests agree on the runtime and permissions.
