# Design Spec: aptupdatechecker Docs Site

## Context

The project has two AsciiDoc files at the repo root (README.adoc for users, DEVELOPMENT.adoc for contributors) that contain solid content but no navigable structure. The goal is to build a proper docs site with separate user and contributor sections, hosted in a `docs/` subdirectory of the same repo. The platform is undecided but content will be AsciiDoc throughout (natural fit for Antora). Doc comment extraction from source is deferred; module pages will be hand-written.

## Directory Structure

```
docs/
  modules/
    user/
      nav.adoc
      pages/
        overview.adoc
        how-it-works.adoc
        installation.adoc
        commands.adoc
        schedule.adoc
        configuration.adoc
        troubleshooting.adoc
    contributor/
      nav.adoc
      pages/
        getting-started.adoc
        architecture.adoc
        module-main.adoc
        module-common.adoc
        module-apt-update.adoc
        module-fw-update.adoc
        testing.adoc
        ci-and-packaging.adoc
        releasing.adoc
  antora.yml
```

README.adoc and DEVELOPMENT.adoc stay at the repo root unchanged, with a short note pointing to the docs site.

## Page Inventory

### User Section (7 pages)

| Page | Source | Notes |
|------|--------|-------|
| overview.adoc | README.adoc | What it is, Debian testing compatibility, key features at a glance |
| how-it-works.adoc | README + DEVELOPMENT | Expanded: privilege separation model, systemd units as root vs. user session, D-Bus notification flow |
| installation.adoc | README.adoc | Prerequisites, build .deb, install, enable user units, verify it's running |
| commands.adoc | README.adoc | Reference for `update`, `check`, `fwupd` subcommands |
| schedule.adoc | README.adoc | Timer table with explanation of first-run timing and frequency |
| configuration.adoc | New content | Customizing schedules via systemd drop-in overrides; what is and isn't configurable |
| troubleshooting.adoc | New content | Notifications not appearing, timers not firing, reading journalctl for this service |

### Contributor Section (9 pages)

| Page | Source | Notes |
|------|--------|-------|
| getting-started.adoc | DEVELOPMENT.adoc | Rust toolchain, cargo-deb, just, first build, running tests |
| architecture.adoc | DEVELOPMENT.adoc | Design decisions, privilege separation rationale, systemd unit map |
| module-main.adoc | New (from src/main.rs) | Entry point: CLI dispatch, subcommand routing |
| module-common.adoc | New (from src/common.rs) | Shared notification utilities, D-Bus session detection, ~216 lines |
| module-apt-update.adoc | New (from src/apt_update.rs) | APT cache update logic, staleness checks, notification dispatch, ~318 lines |
| module-fw-update.adoc | New (from src/fw_update.rs) | fwupd invocation, output parsing via regex, notification dispatch, ~250 lines |
| testing.adoc | DEVELOPMENT.adoc | Unit tests, property-based tests (proptest), coverage (tarpaulin) |
| ci-and-packaging.adoc | DEVELOPMENT.adoc | GitHub Actions workflows, Docker CI image, cargo-deb, postinst |
| releasing.adoc | DEVELOPMENT.adoc | Version bump process, CI release pipeline |

## Content Notes

- **configuration.adoc** is the most speculative new page: the tool has no config file; configuration is entirely via systemd drop-in overrides. The page should explain this pattern and show a concrete example override for changing the APT check frequency.
- **troubleshooting.adoc** should cover: confirming systemd user units are enabled, checking `journalctl --user -u aptupdatechecker_*`, verifying D-Bus is accessible in the session, and what notification daemon is required.
- **Module pages** should be narrative (purpose, key design decisions, data flow) rather than API reference. They should read the source files and explain the non-obvious parts.

## Deferred

- Auto-generation of doc comment content from source (rustdoc JSON or syn-based extraction). Revisit when the docs site toolchain is chosen.
- Versioned docs (Antora supports it; defer until the project reaches 1.0).

## Verification

1. All 16 pages exist under `docs/modules/`
2. nav.adoc files correctly enumerate all pages in each module
3. Existing content pages accurately reflect the current README.adoc and DEVELOPMENT.adoc (no stale information)
4. New pages (configuration, troubleshooting, four module pages) reviewed for accuracy against the source files and systemd units
5. README.adoc and DEVELOPMENT.adoc each have a note pointing to the docs site
6. antora.yml (or equivalent) is present and valid for the chosen platform

## Implementation Notes

- Write the four module pages by reading src/*.rs directly; pay attention to the inline doc comments already present
- The `just` recipes in the justfile are the canonical commands reference for getting-started.adoc
- The systemd unit files in systemd/system/ and systemd/user/ are the canonical source for schedule.adoc and architecture.adoc
