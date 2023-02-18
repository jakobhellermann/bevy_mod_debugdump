#!/bin/bash

set -eu

REPO_BASE="https://raw.githubusercontent.com/jakobhellermann/bevy_mod_debugdump/bevy-main"
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


truncate -s 0 by-crate/README.md
for path in by-crate/light/schedule*.svg; do
    file=$(basename "$path")
    name=$(echo "$file" | sed 's|schedule_Main_\(.*\).dot.svg|\1|')
    printf "## $name\n\n" >> by-crate/README.md
    picture "$name" docs/schedule/by-crate/{light,dark}/"$file" >> by-crate/README.md
done
popd >/dev/null
