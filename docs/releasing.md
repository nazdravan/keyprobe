# Releasing KeyProbe

Releases are made **automatically** by GitHub Actions (`.github/workflows/release.yml`) on every
push to `main`:

- the tests run (`cargo xtask ci`);
- if the version in `Cargo.toml` is **not published yet**, every platform is built and the release
  "KeyProbe v<version>" is published, with the tag `v<version>`, `SHA256SUMS.txt` and the
  version's notes from `CHANGELOG.md`, followed by GitHub's generated list of commits. The release
  is created as a draft, gets all its files, and only then becomes public;
- if the version is already published, nothing is built: an ordinary push doesn't make a release.
  A published release is never overwritten;
- if a build or the tests fail, nothing is published;
- a version with a suffix (`0.2.0-rc.1`) is published as a *pre-release*.

Started by hand (*Actions → Release → Run workflow*), the workflow builds and tests any branch and
keeps the packages as run artifacts, without a release. An optional version input
(`0.2.0-rc.1`) is applied for that run only.

## The version

The version lives in one place: `version` in the `[workspace.package]` section of `Cargo.toml`.

```sh
packaging/version.sh              # print it
packaging/version.sh set 0.2.0    # change it in Cargo.toml and Cargo.lock
```

## The changelog

Each version has a section `## [0.2.0] — <date>` in `CHANGELOG.md`, with *Added*, *Changed* and
*Fixed*. It becomes the release notes; without it the workflow stops before the builds.

```sh
packaging/changelog.sh            # the notes of the version in Cargo.toml
packaging/changelog.sh 0.2.0      # another version's notes
```

## Steps

1. On `main`, raise the version, write its changelog section, commit and push:
   ```sh
   packaging/version.sh set 0.2.0
   git commit -am "Version 0.2.0"
   git push
   ```
2. Follow the **Release** workflow in the *Actions* tab. When it ends, the release is published
   under *Releases*.

If a build failed, fix it and push to `main` again with the same version: as long as the version
isn't published, every push retries the release. If a published release has a mistake, make a new
version (`0.2.1`).

**The release commit must not change files in `.github/workflows/`.** The workflow's token may not
create tags on such commits, and publishing fails with `HTTP 403: Resource not accessible by
integration`. Push workflow changes in an earlier commit, then the version bump.

## What gets built

| File | Platform |
|---|---|
| `keyprobe-<version>-linux-x86_64.AppImage` | Linux: `chmod +x` and run |
| `keyprobe-<version>-linux-x86_64.deb` | Debian / Ubuntu: `sudo apt install ./keyprobe-*.deb` |
| `keyprobe-<version>-linux-x86_64.tar.gz` | Linux: the binary and licences |
| `keyprobe-<version>-windows-x64.msi` | Windows 10/11: installer |
| `keyprobe-<version>-windows-x64-portable.zip` | Windows 10/11: no install |
| `keyprobe-<version>-macos-universal.dmg` | macOS 10.13+, Apple Silicon and Intel, ad-hoc signed |
| `SHA256SUMS.txt` | checksums of all of the above |

Linux packages are built on Ubuntu 22.04, so they need glibc 2.35 or newer. The binaries are not
signed with a paid certificate: Windows SmartScreen and macOS Gatekeeper warn on first start (see
the README's Quick start).

Every script in `packaging/` also runs locally on its own platform, e.g.
`packaging/macos/package.sh` on a Mac; the files land in `dist/release/`.

## Licences in the packages

Every package carries `LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE`, `THIRD-PARTY.txt` and the font
licences in `licenses/`. The packaging scripts copy them only through `copy_docs`
(`packaging/env.sh`; in `package.ps1`, `Copy-Docs`).

- `THIRD-PARTY.txt` is generated, not in git: `packaging/licences.sh` (needs
  `cargo install cargo-about --locked --features cli --version <CARGO_ABOUT_VERSION from env.sh>`)
  writes it to `dist/` with the licence of every crate and of egui's fonts. In CI the `licences`
  job makes it on every run, and fails if a crate has a licence that is not in `about.toml`.
- After an egui update the checksums in `[epaint_default_fonts.clarify]` (`about.toml`) stop
  matching and `packaging/licences.sh` stops: check the crate's `fonts/*.txt` and update the sums
  (`shasum -a 256`).
- A new file in `assets/fonts/` must be named in `NOTICE`, with its licence: otherwise
  `cargo xtask licences` (part of `cargo xtask ci`) fails.
