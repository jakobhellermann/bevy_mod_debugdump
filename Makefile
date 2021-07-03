view: render_graph.svg render_schedule_graph.svg schedule_graph.svg
	firefox render_graph.svg
	firefox render_schedule_graph.svg
	firefox schedule_graph.svg

render_graph.svg:
	cargo run --example print_render_graph | dot -Tsvg -o render_graph.svg

render_schedule_graph.svg:
	cargo run --example print_render_schedule_graph | dot -Tsvg -o render_schedule_graph.svg

schedule_graph.svg:
	cargo run --example print_schedule_graph | dot -Tsvg -o schedule_graph.svg

make clean:
	rm *.{svg,png}