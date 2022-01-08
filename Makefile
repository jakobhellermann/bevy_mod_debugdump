.PHONY: docs
docs: examples docs/render_graph.svg docs/render_schedule_graph.svg docs/schedule_graph.svg

.PHONY: docs_png
docs_png: examples
	cargo run -q --example print_render_graph          | dot -Tpng -Nfontname=Roboto -o docs/render_graph.png
	cargo run -q --example print_render_schedule_graph | dot -Tpng -Nfontname=Roboto -o docs/render_schedule_graph.png
	cargo run -q --example print_schedule_graph        | dot -Tpng -Nfontname=Roboto -o docs/schedule_graph.png

.PHONY: examples
examples:
	cargo build --examples

.PHONY: view
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
