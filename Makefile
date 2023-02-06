REPO_BASE=https://raw.githubusercontent.com/jakobhellermann/bevy_mod_debugdump/stageless

.PHONY: docs
docs:
	cargo run --example generate_docs
	ls docs/{light,dark}/*.dot docs/by-crate/{light,dark}/*.dot | xargs -I '{}' dot -Tsvg '{}' -o '{}.svg'

	./docs/generate_readme.sh

.PHONY: compare
compare:
	cargo run --example compare
	ls compare/*.dot | xargs -I '{}' dot -Tsvg '{}' -o '{}.svg'