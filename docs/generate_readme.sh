#!/bin/bash

set -eu

REPO_BASE="https://raw.githubusercontent.com/jakobhellermann/bevy_mod_debugdump/stageless"
cd $(dirname "${BASH_SOURCE[0]}")

printf "# Main App\n\n" > README.md
ls schedule*.svg | sd 'schedule_(.*).dot.svg' "## \$1\n\n![\$1]($REPO_BASE/docs/schedule_\$1.dot.svg)\n" \
    >> README.md

printf "# Render App\n\n" >> README.md
ls render_*.svg | sd 'render_schedule_(.*).dot.svg' "## \$1\n\n![\$1]($REPO_BASE/docs/render_schedule_\$1.dot.svg)\n" \
    >> README.md

ls by-crate/*.svg | sd 'by-crate/schedule_Main_(.*).dot.svg' "# \$1\n\n![\$1]($REPO_BASE/docs/by-crate/schedule_Main_\$1.dot.svg)\n" > by-crate/README.md