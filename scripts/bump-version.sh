#!/usr/bin/env bash
# Bump the package version in Cargo.toml and Cargo.lock, commit on a
# `bump-vX.Y.Z` branch, and push. Tagging happens on PR merge via CI.

set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/bump-version.sh <major|minor|patch> [--no-push] [--allow-dirty]

Arguments:
  major|minor|patch   Which part of the semver to bump.

Options:
  --no-push           Do not push the branch (useful for local iteration).
  --allow-dirty       Skip the working-tree-clean check (used by CI).
  -h, --help          Show this help.

Exit codes:
  0  Success
  1  Usage error or missing files
  2  Precondition failure (dirty tree, wrong branch, branch exists)
  3  Internal failure (edit produced no change, cargo check failed)
EOF
}

BUMP=""
ALLOW_DIRTY=0
NO_PUSH=0

while [ $# -gt 0 ]; do
  case "$1" in
    --no-push) NO_PUSH=1 ;;
    --allow-dirty) ALLOW_DIRTY=1 ;;
    -h|--help) usage; exit 0 ;;
    major|minor|patch)
      if [ -n "$BUMP" ]; then
        echo "Error: bump type specified twice" >&2
        exit 1
      fi
      BUMP="$1"
      ;;
    *)
      echo "Error: unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
  shift
done

if [ -z "$BUMP" ]; then
  echo "Error: bump type required (major|minor|patch)" >&2
  usage >&2
  exit 1
fi

if [ ! -f Cargo.toml ]; then
  echo "Error: Cargo.toml not found in $(pwd)" >&2
  exit 1
fi

# Precondition: must be inside a git repo.
if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "Error: not inside a git repository" >&2
  exit 2
fi

# Precondition: working tree must be clean unless --allow-dirty.
if [ "$ALLOW_DIRTY" -eq 0 ]; then
  if ! git diff --quiet || ! git diff --cached --quiet; then
    echo "Error: working tree has uncommitted changes. Commit, stash, or pass --allow-dirty." >&2
    exit 2
  fi
fi

# Precondition: must be on main.
CURRENT_BRANCH="$(git rev-parse --abbrev-ref HEAD)"
if [ "$CURRENT_BRANCH" != "main" ]; then
  echo "Error: current branch is '$CURRENT_BRANCH', expected 'main'" >&2
  exit 2
fi

# Pull latest main with fast-forward only. Skip if --allow-dirty (CI's
# checkout is already at the desired ref and may be in detached state).
if [ "$ALLOW_DIRTY" -eq 0 ]; then
  git pull --ff-only origin main
fi

# Read the version from the [package] section of Cargo.toml.
get_current_version() {
  awk '
    /^\[/ { in_package = ($0 == "[package]") }
    in_package && /^version[[:space:]]*=/ {
      gsub(/^version[[:space:]]*=[[:space:]]*"/, "")
      gsub(/".*/, "")
      print
      exit
    }
  ' Cargo.toml
}

# Read the crate name from the [package] section of Cargo.toml.
get_crate_name() {
  awk '
    /^\[/ { in_package = ($0 == "[package]") }
    in_package && /^name[[:space:]]*=/ {
      gsub(/^name[[:space:]]*=[[:space:]]*"/, "")
      gsub(/".*/, "")
      print
      exit
    }
  ' Cargo.toml
}

# Compute the next semver given the current version and bump type.
# Args: $1 = current version (e.g. 1.2.3), $2 = bump type (major|minor|patch)
compute_next_version() {
  local current="$1"
  local bump="$2"
  local major minor patch
  IFS='.' read -r major minor patch <<< "$current"
  case "$bump" in
    major) echo "$((major + 1)).0.0" ;;
    minor) echo "${major}.$((minor + 1)).0" ;;
    patch) echo "${major}.${minor}.$((patch + 1))" ;;
  esac
}

CURRENT_VERSION="$(get_current_version)"
if [ -z "$CURRENT_VERSION" ]; then
  echo "Error: no version line found in [package] section of Cargo.toml" >&2
  exit 1
fi

if ! [[ "$CURRENT_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Error: current version '$CURRENT_VERSION' is not semver" >&2
  exit 1
fi

CRATE_NAME="$(get_crate_name)"
if [ -z "$CRATE_NAME" ]; then
  echo "Error: no name line found in [package] section of Cargo.toml" >&2
  exit 1
fi

NEW_VERSION="$(compute_next_version "$CURRENT_VERSION" "$BUMP")"
NEW_TAG="v${NEW_VERSION}"

echo "current version: $CURRENT_VERSION"
echo "new version: $NEW_VERSION"
echo "new tag: $NEW_TAG"

# Replace the version line in the [package] section of Cargo.toml.
update_cargo_toml() {
  local new_version="$1"
  awk -v ver="$new_version" '
    /^\[/ { in_package = ($0 == "[package]") }
    in_package && /^version[[:space:]]*=/ {
      print "version = \"" ver "\""
      next
    }
    { print }
  ' Cargo.toml > Cargo.toml.tmp
  mv Cargo.toml.tmp Cargo.toml
}

update_cargo_toml "$NEW_VERSION"

# Verify the edit took effect.
if [ "$(get_current_version)" != "$NEW_VERSION" ]; then
  echo "Error: Cargo.toml edit did not take effect" >&2
  exit 3
fi
echo "updated Cargo.toml to version $NEW_VERSION"

# Regenerate Cargo.lock by running cargo check. Cargo writes Cargo.lock on
# any build/check command when the local-package version differs from the
# lockfile entry. Try --offline first to avoid network use.
if ! cargo check --offline >/dev/null 2>&1; then
  cargo check
fi

# Verify Cargo.lock contains the new version for the local crate.
# grep -q closes stdout early, which sends SIGPIPE to the upstream grep under
# pipefail. Use grep -c and discard the count to avoid that.
if ! grep -A1 "name = \"$CRATE_NAME\"" Cargo.lock | grep -c "version = \"$NEW_VERSION\"" >/dev/null; then
  echo "Error: Cargo.lock was not updated to version $NEW_VERSION" >&2
  exit 3
fi
echo "updated Cargo.lock to version $NEW_VERSION"

BUMP_BRANCH="bump-${NEW_TAG}"

# Fail fast if the bump branch already exists locally or remotely.
if git show-ref --verify --quiet "refs/heads/$BUMP_BRANCH"; then
  echo "Error: branch '$BUMP_BRANCH' already exists locally" >&2
  exit 2
fi
if git ls-remote --exit-code --heads origin "$BUMP_BRANCH" >/dev/null 2>&1; then
  echo "Error: branch '$BUMP_BRANCH' already exists on origin" >&2
  exit 2
fi

git switch -c "$BUMP_BRANCH"
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to ${NEW_TAG}"

if [ "$NO_PUSH" -eq 0 ]; then
  git push --set-upstream origin "$BUMP_BRANCH"
  echo "pushed branch $BUMP_BRANCH"
else
  echo "skipped push (--no-push)"
fi

# Final line: just the new tag, for callers that want to capture it.
echo "$NEW_TAG"
