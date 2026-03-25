<!--
SPDX-License-Identifier: AGPL-3.0-or-later
Documentation and creative text in this file: CC-BY-SA-4.0
-->

# plasmidBin — Public Binary Distribution Strategy

**Status**: Active
**Date**: March 24, 2026
**Repository**: [github.com/ecoPrimals/plasmidBin](https://github.com/ecoPrimals/plasmidBin)
**Related**: [VISION_AND_EVOLUTION.md](VISION_AND_EVOLUTION.md),
[ESOTERIC_WEBB_DESIGN.md](ESOTERIC_WEBB_DESIGN.md),
[EVOLUTION_GAPS.md](../EVOLUTION_GAPS.md) (GAP-010)

---

## Problem

The ecoPrimals ecosystem has two visibility tiers:

- **Public**: springs (ecoSprings/) and consumers (esotericWebb) — open
  source, anyone can clone and build.
- **Private**: primal source repos (phase1/, phase2/, standalone primals) —
  evolving toward publication but not yet ready.

Consumers need primal binaries to exercise composition. Without binaries,
Webb runs in standalone degradation mode — functional but unable to
demonstrate the full stack. Anyone cloning the public repos has no way to
get working primal binaries.

The springs and consumers are public. The primals are private. The
**binaries** need to be public.

---

## Solution: metadata in git, binaries in GitHub Releases

`ecoPrimals/plasmidBin` is a public repository that separates concerns:

- **Git tracks metadata**: `metadata.toml` per primal (version, checksum,
  capabilities, provenance), `manifest.lock` (pinned deployment state),
  scripts, and documentation.
- **GitHub Releases hold binaries**: compiled primal binaries are uploaded
  as release assets, tagged with date-based versions. Releases are free,
  unlimited size, versioned, and downloadable with `gh release download`
  or `curl`.
- **Binaries are excluded from git**: `.gitignore` prevents committing
  binaries to the repository. Git history stays small. Binaries are
  ephemeral artifacts, not versioned source.

```
plasmidBin/ (git repo)
  README.md                    tracked
  SOURCE_AVAILABILITY.md       tracked — AGPL compliance
  manifest.lock                tracked — pinned versions
  harvest.sh                   tracked — build + release script
  fetch.sh                     tracked — download + verify script
  .gitignore                   tracked — excludes binaries
  rhizocrypt/
    metadata.toml              tracked — version, checksum, capabilities
    rhizocrypt                 NOT tracked — downloaded from Release
  loamspine/
    metadata.toml              tracked
    loamspine                  NOT tracked
  sweetgrass/
    metadata.toml              tracked
    sweetgrass                 NOT tracked
```

```
GitHub Releases (ecoPrimals/plasmidBin)
  v2026.03.24 — "Provenance trio harvest"
    rhizocrypt    (5.7 MB)
    loamspine     (8.3 MB)
    sweetgrass    (12 MB)
  v2026.04.xx — future harvest
    ...additional primals
```

---

## Why GitHub Releases

| Approach | Pros | Cons | Verdict |
|----------|------|------|---------|
| **GitHub Releases** | Free, unlimited, versioned, `gh` CLI, standard | Two-step (commit metadata, create release) | Recommended |
| **Git LFS** | `git clone` gets everything | 1 GB free bandwidth limit, costs past that | Too expensive at scale |
| **Commit binaries** | Simplest workflow | Repo bloats forever, every clone gets every version | Does not scale |
| **External hosting** | Flexible | More moving parts, needs infrastructure | Overkill for now |

GitHub Releases is the right balance: standard tooling, no cost, no repo
bloat, and anyone with `gh` or `curl` can download.

---

## AGPL compliance

All primal binaries are licensed under AGPL-3.0-or-later. AGPL requires
that source code be made available when binaries are distributed.

The `SOURCE_AVAILABILITY.md` file in the plasmidBin repo states:

1. These binaries are AGPL-3.0-or-later.
2. Source code is available on request (email or issue).
3. Primal source repositories will be published when ready.
4. Each `metadata.toml` contains a `built_from` field tracing the binary
   to its source tree.

This satisfies the AGPL's "corresponding source" requirement during the
period between binary publication and source publication. Once primal repos
go public, the `built_from` field becomes a direct link to the source.

---

## Harvest workflow

The maintainer (you) runs `harvest.sh` when new primal binaries are built.

### What harvest.sh does

1. Scans each subdirectory for a `metadata.toml` alongside a binary.
2. Computes SHA-256 checksum of each binary.
3. Updates `metadata.toml` with the new checksum and timestamp.
4. Regenerates `manifest.lock` from all metadata files.
5. Creates a GitHub Release tagged `v<date>` and attaches all binaries.
6. Prints a summary of what was released.

### Manual workflow (without harvest.sh)

```bash
cd plasmidBin/

# 1. Copy fresh binary from private build
cp /path/to/rhizocrypt/target/release/rhizocrypt rhizocrypt/

# 2. Update metadata.toml checksum
sha256sum rhizocrypt/rhizocrypt
# paste into metadata.toml [primal] checksum_sha256 = "..."

# 3. Commit metadata (not binary)
git add rhizocrypt/metadata.toml manifest.lock
git commit -m "harvest: rhizocrypt 0.14.1"

# 4. Create release with binary attached
gh release create v2026.03.25 \
  rhizocrypt/rhizocrypt \
  --title "rhizocrypt 0.14.1" \
  --notes "Updated rhizocrypt to 0.14.1"

git push
```

---

## Fetch workflow

A consumer (anyone cloning a public repo) runs `fetch.sh` to get binaries.

### What fetch.sh does

1. Reads `manifest.lock` for the list of primals and their pinned versions.
2. Downloads binaries from the latest (or specified) GitHub Release.
3. Places each binary in its `<primal>/` directory.
4. Verifies SHA-256 checksums against `metadata.toml`.
5. Makes binaries executable.
6. Reports success or checksum mismatches.

### Manual workflow (without fetch.sh)

```bash
cd plasmidBin/

# Download all assets from latest release
gh release download --pattern '*' --dir .

# Move binaries to their directories
mv rhizocrypt rhizocrypt/
mv loamspine loamspine/
mv sweetgrass sweetgrass/

# Verify checksums
sha256sum -c <<EOF
97a82478...  rhizocrypt/rhizocrypt
186174f8...  loamspine/loamspine
ce448598...  sweetgrass/sweetgrass
EOF

# Make executable
chmod +x */rhizocrypt */loamspine */sweetgrass
```

---

## Consumer integration

### Esoteric Webb

Webb's existing code already consumes plasmidBin with no changes needed:

- **`discover_binary()`** in `webb/src/ipc/launcher.rs` searches
  `$ECOPRIMALS_PLASMID_BIN`, then `./plasmidBin`, `../plasmidBin`, etc.
  for `<primal>/<primal>` binary paths.
- **`discover_from_plasmid_bin()`** in `webb/src/ipc/discovery.rs` reads
  `metadata.toml` files for capability and transport hints.
- **`PrimalLauncher::spawn()`** starts discovered binaries with
  `<binary> server --port <port>` and polls for TCP readiness.

A consumer's workflow:

```bash
# Clone Webb and plasmidBin side by side
git clone git@github.com:sporeGarden/esotericWebb.git
git clone git@github.com:ecoPrimals/plasmidBin.git

# Fetch primal binaries
cd plasmidBin && ./fetch.sh && cd ..

# Webb discovers primals via relative path probe
cd esotericWebb
cargo run -- serve --content content --launch
# [launcher] spawning rhizocrypt from ../../plasmidBin/rhizocrypt/rhizocrypt
# [launcher] rhizocrypt ready at 127.0.0.1:9401 (pid 12345)
```

### Other springs

Springs that need primal capabilities at test time (e.g. ludoSpring testing
against a live Squirrel) follow the same pattern: clone plasmidBin, run
`fetch.sh`, set `ECOPRIMALS_PLASMID_BIN` to point at it, run integration
tests.

### Environment override

For non-standard layouts, set `ECOPRIMALS_PLASMID_BIN`:

```bash
export ECOPRIMALS_PLASMID_BIN=/opt/ecoPrimals/plasmidBin
```

This takes priority over relative path probing.

---

## Version pinning

- **manifest.lock** pins the exact version and checksum of each deployed
  primal. This file is committed to git and tracks what a consumer should
  expect.
- **GitHub Release tags** use date-based versions: `v2026.03.24`,
  `v2026.04.01`, etc. A harvest that updates multiple primals gets a single
  release tag.
- **Per-primal versioning** lives in `metadata.toml` (`[primal] version`).
  A single release can contain primals at different internal versions.
- **Rollback**: pin manifest.lock to a previous commit, run `fetch.sh` with
  an explicit release tag to get older binaries.

---

## Architecture variants

Currently x86_64-linux only (Pop!_OS gate). The `metadata.toml` already has
an `architecture` field. When aarch64-linux or other platforms are needed:

1. Add arch-specific binary names: `rhizocrypt_x86_64_linux`,
   `rhizocrypt_aarch64_linux`
2. `fetch.sh` detects `uname -m` and downloads the matching variant
3. `metadata.toml` gains a `[variants]` table mapping arch to filename
4. `discover_binary()` already searches arch/OS variant patterns

This is a future concern. Start with x86_64-linux and evolve when a second
architecture appears.

---

## Adding a new primal to plasmidBin

When a new primal binary is ready for public consumption:

1. Create the directory: `mkdir -p <primal>/`
2. Copy the binary: `cp /path/to/target/release/<primal> <primal>/`
3. Create `metadata.toml` following the existing pattern (see
   `rhizocrypt/metadata.toml` for reference)
4. Run `harvest.sh` to update checksums, manifest, and create a release
5. Commit and push

---

## Relationship to ecosystem standards

This strategy implements the patterns described in:

- **wateringHole `UNIBIN_ARCHITECTURE_STANDARD.md`**: one binary per primal,
  subcommands for modes
- **wateringHole `ECOBIN_ARCHITECTURE_STANDARD.md`**: pure Rust, platform-
  agnostic IPC, cross-compilation ready
- **wateringHole `GENOMEBIN_ARCHITECTURE_STANDARD.md`**: deployment wrapper
  with OS integration (future: `fetch.sh` evolves toward genomeBin install)
- **plasmidBin README**: central deployment surface, manifest registry,
  consumer discovery patterns

The public plasmidBin repo is the concrete implementation of what
wateringHole describes in the abstract.
