#!/usr/bin/env sh
set -eu

script_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
guard="$script_root/publication-guard.sh"
test_repository=$(mktemp -d)
trap 'rm -rf "$test_repository"' EXIT HUP INT TERM

git -C "$test_repository" init --quiet
git -C "$test_repository" config user.name "Publication Guard Test"
git -C "$test_repository" config user.email "1+guard-test@users.noreply.github.com"
printf 'safe\n' >"$test_repository/evidence.txt"
git -C "$test_repository" add evidence.txt
git -C "$test_repository" -c commit.gpgsign=false commit --quiet -m "test: safe identity"

(cd "$test_repository" && "$guard" HEAD) >/dev/null

git -C "$test_repository" config user.email "private@example.invalid"
printf 'private\n' >>"$test_repository/evidence.txt"
git -C "$test_repository" add evidence.txt
git -C "$test_repository" -c commit.gpgsign=false commit --quiet -m "test: private identity"

if (cd "$test_repository" && "$guard" HEAD) >/dev/null 2>&1; then
  echo "publication guard regression: private identity unexpectedly passed" >&2
  exit 1
fi

mkdir -p "$test_repository/.github"
printf 'private@example.invalid\n' >"$test_repository/.github/public-emails.txt"
git -C "$test_repository" add .github/public-emails.txt
git -C "$test_repository" -c commit.gpgsign=false commit --quiet -m "test: declare public identity"

(cd "$test_repository" && "$guard" HEAD) >/dev/null
echo "publication guard regression: pass"
