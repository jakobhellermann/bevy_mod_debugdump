# bevy_mod_debugdump
![Crates.io](https://img.shields.io/crates/v/bevy_mod_debugdump)
![Crates.io](https://img.shields.io/crates/d/bevy_mod_debugdump)

Live playground: [jakobhellermann.github.io/bevy_mod_debugdump](https://jakobhellermann.github.io/bevy_mod_debugdump)

## Schedule graph

```rust
use bevy::prelude::*;
use bevy::log::LogPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.build().disable::<LogPlugin>()); // disable LogPlugin so that you can pipe the output directly into `dot -Tsvg`
    bevy_mod_debugdump::print_schedule_graph(&mut app, Update);
}
```

`PreUpdate` schedule:

<picture>
<source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/jakobhellermann/bevy_mod_debugdump/main/docs/schedule/dark/schedule_PreUpdate.dot.svg">
<img alt="Main (filtered)" src="https://raw.githubusercontent.com/jakobhellermann/bevy_mod_debugdump/main/docs/schedule/light/schedule_PreUpdate.dot.svg">
</picture>

See all schedules at [docs/schedule](./docs/schedule/README.md).

## Render app

### Render graph

```rust
use bevy::prelude::*;
use bevy::log::LogPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.build().disable::<LogPlugin>()); 
    bevy_mod_debugdump::print_render_graph(&mut app);
}
```

<picture>
<source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/jakobhellermann/bevy_mod_debugdump/bevy/docs/render/dark/render_graph.dot.svg">
<img alt="render graph" src="https://raw.githubusercontent.com/jakobhellermann/bevy_mod_debugdump/bevy/docs/render/light/render_graph.dot.svg">
</picture>

### Extract schedule

<picture>
<source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/jakobhellermann/bevy_mod_debugdump/bevy-main/docs/schedule/dark/render_schedule_ExtractSchedule.dot.svg">
<img alt="ExtractSchedule" src="https://raw.githubusercontent.com/jakobhellermann/bevy_mod_debugdump/bevy-main/docs/schedule/light/render_schedule_ExtractSchedule.dot.svg" height=512>
</picture>

### Main render schedule

<picture>
<source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/jakobhellermann/bevy_mod_debugdump/bevy-main/docs/schedule/dark/render_schedule_Render.dot.svg">
<img alt="Main" src="https://raw.githubusercontent.com/jakobhellermann/bevy_mod_debugdump/bevy-main/docs/schedule/light/render_schedule_Render.dot.svg">
</picture>


## Bevy support table

|bevy|bevy\_mod\_debugdump|
|---|---|
|0.12|0.9|
|0.11|0.8|
|0.10|0.7|
|0.9|0.6|
|0.8|0.5|
|0.7|0.4|
|0.6|0.3|
|0.5|0.2|
|0.5|0.1|
