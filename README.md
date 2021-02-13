# bevy_mod_debugdump

## Features
- print out bevy's render graph:
```rust
use bevy::prelude::*;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_system(debug.system())
        .run();
}

fn print_render_graph(render_graph: Res<RenderGraph>) {
    let dot = render_graph_dot(&*render_graph);
    println!("{}", dot);
}
```

![bevy's render graph](docs/render_graph.png)