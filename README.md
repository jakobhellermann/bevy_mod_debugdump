# bevy_mod_debugdump

## Features
### Schedule graph
```rust
use bevy::prelude::*;
use bevy_mod_debugdump::schedule_graph_dot;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    bevy_mod_debugdump::print_schedule(&mut app);
}
```

![bevy's schedule graph](https://raw.githubusercontent.com/jakobhellermann/bevy_mod_debugdump/main/docs/schedule_graph.svg)

### Render Graph
```rust
use bevy::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
    bevy_mod_debugdump::print_render_graph(&mut app);
}

fn print_render_graph(render_graph: Res<RenderGraph>) {
    let dot = render_graph_dot(&*render_graph);
    println!("{}", dot);
}
```

![bevy's render graph](https://raw.githubusercontent.com/jakobhellermann/bevy_mod_debugdump/main/docs/render_graph.svg)

**Render schedule graph**
![bevy's render schedule graph](https://raw.githubusercontent.com/jakobhellermann/bevy_mod_debugdump/main/docs/render_schedule_graph.svg)

## Bevy support table

|bevy|bevy_mod_debugdump|
|---|---|
|0.6|0.3|
|0.5|0.2|
|0.5|0.1|
