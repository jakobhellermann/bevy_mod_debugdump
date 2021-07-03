view: examples render_graph.svg render_schedule_graph.svg schedule_graph.svg
	firefox render_graph.svg
	firefox render_schedule_graph.svg
	firefox schedule_graph.svg

render_graph.svg:
	cargo run -q --example print_render_graph | dot -Tsvg -o render_graph.svg

render_schedule_graph.svg:
	cargo run -q --example print_render_schedule_graph | dot -Tsvg -o render_schedule_graph.svg

schedule_graph.svg:
	cargo run -q --example print_schedule_graph | dot -Tsvg -o schedule_graph.svg

.PHONY: examples
examples:
	cargo build --examples

make clean:
	rm -f *.{svg,png}
