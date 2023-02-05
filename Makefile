.PHONY: docs
docs:
	cargo run --example generate_docs
	ls docs/*.dot docs/*/*.dot | xargs -I '{}' dot -Tsvg '{}' -o '{}.svg'

.PHONY: compare
compare:
	cargo run --example compare
	ls compare/*.dot | xargs -I '{}' dot -Tsvg '{}' -o '{}.svg'