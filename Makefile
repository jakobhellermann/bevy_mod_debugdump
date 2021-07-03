.PHONY: docs
docs: examples docs/render_graph.svg docs/render_schedule_graph.svg docs/schedule_graph.svg

.PHONY: examples
examples:
	cargo build --examples

view: docs
	firefox docs/render_graph.svg
	firefox docs/render_schedule_graph.svg
	firefox docs/schedule_graph.svg

docs/render_graph.svg:
	cargo run -q --example print_render_graph | dot -Tsvg -o docs/render_graph.svg

docs/render_schedule_graph.svg:
	cargo run -q --example print_render_schedule_graph | dot -Tsvg -o docs/render_schedule_graph.svg

docs/schedule_graph.svg:
	cargo run -q --example print_schedule_graph | dot -Tsvg -o docs/schedule_graph.svg

make clean:
	@rm -f *.svg
	@rm -f *.png
	@rm -f *.dot
