#!/usr/bin/env sh
set -eu

ref="${1:-HEAD}"
allow_dirty="${2:-}"
repository_root=$(git rev-parse --show-toplevel)
cd "$repository_root"

git rev-parse --verify "$ref^{commit}" >/dev/null

if [ "$allow_dirty" != "--allow-dirty" ] && [ -n "$(git status --porcelain)" ]; then
  echo "publication guard: working tree is not clean" >&2
  exit 1
fi

allowed_file="$repository_root/.github/public-emails.txt"
temporary_records=$(mktemp)
trap 'rm -f "$temporary_records"' EXIT HUP INT TERM
git log "$ref" --format='%H%x09%ae%x09%ce' >"$temporary_records"

is_allowed_email() {
  candidate=$1
  if printf '%s\n' "$candidate" | grep -Eq '^([0-9]+\+)?[A-Za-z0-9][A-Za-z0-9-]{0,38}@users\.noreply\.github\.com$'; then
    return 0
  fi
  if [ -f "$allowed_file" ] && grep -Fxiq -- "$candidate" "$allowed_file"; then
    return 0
  fi
  return 1
}

failed=0
tab=$(printf '\t')
while IFS="$tab" read -r commit author_email committer_email; do
  if ! is_allowed_email "$author_email"; then
    echo "publication guard: $commit has an unapproved author email: $author_email" >&2
    failed=1
  fi
  if ! is_allowed_email "$committer_email"; then
    echo "publication guard: $commit has an unapproved committer email: $committer_email" >&2
    failed=1
  fi
done <"$temporary_records"

configured_email=$(git config --get user.email || true)
if [ -z "$configured_email" ] || ! is_allowed_email "$configured_email"; then
  echo "publication guard: repository user.email is missing or is not approved for public history" >&2
  failed=1
fi

if [ "$failed" -ne 0 ]; then
  echo "Use a GitHub noreply address or list an intentionally public address in .github/public-emails.txt." >&2
  exit 1
fi

commit_count=$(wc -l <"$temporary_records" | tr -d ' ')
printf '{"ok":true,"ref":"%s","commits":%s}\n' "$ref" "$commit_count"
