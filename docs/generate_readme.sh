#!/bin/bash

set -eu

REPO_BASE="https://raw.githubusercontent.com/jakobhellermann/bevy_mod_debugdump/stageless-themed"
cd $(dirname "${BASH_SOURCE[0]}")

printf "# Main App\n\n" > README.md
ls light/schedule*.svg | sd 'light/schedule_(.*).dot.svg' "## \$1\n\n![\$1]($REPO_BASE/docs/light/schedule_\$1.dot.svg)\n" \
    >> README.md

printf "# Render App\n\n" >> README.md
ls light/render_*.svg | sd 'light/render_schedule_(.*).dot.svg' "## \$1\n\n![\$1]($REPO_BASE/docs/light/render_schedule_\$1.dot.svg)\n" \
    >> README.md

ls by-crate/light/*.svg | sd 'by-crate/light/schedule_Main_(.*).dot.svg' "# \$1\n\n![\$1]($REPO_BASE/docs/by-crate/light/schedule_Main_\$1.dot.svg)\n" > by-crate/README.md