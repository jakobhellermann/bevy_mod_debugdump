# bevy_mod_debugdump

## Features
### Schedule graph
```rust
use bevy::prelude::*;
use bevy::log::LogPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins_with(DefaultPlugins, |plugins| plugins.disable::<bevy::log::LogPlugin>()); // disable LogPlugin so that you can pipe the output directly into `dot -Tsvg`
    bevy_mod_debugdump::print_schedule(&mut app);
}
```

![bevy's schedule graph](https://raw.githubusercontent.com/jakobhellermann/bevy_mod_debugdump/main/docs/schedule_graph.svg)

### Render Graph
```rust
use bevy::prelude::*;
use bevy::log::LogPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins_with(DefaultPlugins, |plugins| plugins.disable::<LogPlugin>());
    bevy_mod_debugdump::print_render_graph(&mut app);
}
```

![bevy's render graph](https://raw.githubusercontent.com/jakobhellermann/bevy_mod_debugdump/main/docs/render_graph.svg)

**Render schedule graph**

```rust
use bevy::prelude::*;
use bevy::log::LogPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins_with(DefaultPlugins, |plugins| plugins.disable::<LogPlugin>());
    bevy_mod_debugdump::print_render_schedule(&mut app);
}
```

![bevy's render schedule graph](https://raw.githubusercontent.com/jakobhellermann/bevy_mod_debugdump/main/docs/render_schedule_graph.svg)

## Bevy support table

|bevy|bevy_mod_debugdump|
|---|---|
|0.6|0.3|
|0.5|0.2|
|0.5|0.1|
