.PHONY: docs
docs:
	cargo run --example generate_docs
	ls docs/schedule/{light,dark}/*.dot docs/render/{light,dark}/*.dot | xargs -I '{}' dot -Tsvg '{}' -o '{}.svg'

	./docs/generate_readme.sh

.PHONY: compare
compare:
	cargo run --example compare
	ls compare/*.dot | xargs -I '{}' dot -Tsvg '{}' -o '{}.svg'
