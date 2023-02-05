REPO_BASE=https://raw.githubusercontent.com/jakobhellermann/bevy_mod_debugdump/stageless

.PHONY: docs
docs:
	cargo run --example generate_docs
	ls docs/*.dot docs/*/*.dot | xargs -I '{}' dot -Tsvg '{}' -o '{}.svg'

	ls docs/*.svg | sd 'docs/schedule_(.*).dot.svg' '# $$1\n\n![$$1](${REPO_BASE}/docs/schedule_$$1.dot.svg)\n' > docs/README.md
	ls docs/by-crate/*.svg | sd 'docs/by-crate/schedule_Main_(.*).dot.svg' '# $$1\n\n![$$1](${REPO_BASE}/docs/by-crate/schedule_Main_$$1.dot.svg)\n' > docs/by-crate/README.md

.PHONY: compare
compare:
	cargo run --example compare
	ls compare/*.dot | xargs -I '{}' dot -Tsvg '{}' -o '{}.svg'