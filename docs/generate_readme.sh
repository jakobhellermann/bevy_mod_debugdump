#!/bin/bash

set -eu

REPO_BASE="https://raw.githubusercontent.com/jakobhellermann/bevy_mod_debugdump/main"
cd $(dirname "${BASH_SOURCE[0]}")

function picture() {
    printf '<picture>\n'
    printf "<source media=\"(prefers-color-scheme: dark)\" srcset=\"$REPO_BASE/$3\">\n"
    printf "<img alt=\"$1\" src=\"$REPO_BASE/$2\">\n"
    printf '</picture>\n\n'
}

pushd schedule >/dev/null
printf "# Main App\n\n" > README.md
for path in light/schedule*.svg; do
    file=$(basename "$path")
    name=$(echo "$file" | sed 's|schedule_\(.*\).dot.svg|\1|' | tr '_' ' ')
    printf "## $name\n\n" >> README.md
    picture "$name" docs/schedule/{light,dark}/"$file" >> README.md
done

printf "# Render App\n\n" >> README.md
for path in light/render_schedule*.svg; do
    file=$(basename "$path")
    name=$(echo "$file" | sed 's|render_schedule_\(.*\).dot.svg|\1|' | tr '_' ' ')
    printf "## $name\n\n" >> README.md
    picture "$name" docs/schedule/{light,dark}/"$file" >> README.md
done