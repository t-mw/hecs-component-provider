# hecs-component-provider

[![Crates.io](https://img.shields.io/crates/v/hecs-component-provider.svg)](https://crates.io/crates/hecs-component-provider)
[![Docs Status](https://docs.rs/hecs-component-provider/badge.svg)](https://docs.rs/hecs-component-provider)

Easily define behavior for sets of components when using the hecs ECS library.

```rust
use hecs_component_provider::{
    default_trait_impl, gen_tuple_query_component_providers,
    ComponentProvider, ComponentProviderMut
};

struct Position(f32, f32);
struct Velocity(f32, f32);

// implement a behavior for all entities that provide the required components
#[default_trait_impl]
trait Update: ComponentProviderMut<Position> + ComponentProvider<Velocity> {
    fn update(&mut self, dt: f32) {
        let &Velocity(vx, vy) = self.get();
        let position: &mut Position = self.get_mut();
        position.0 += vx * dt;
        position.1 += vy * dt;
    }
}

let mut world = hecs::World::new();
world.spawn((Position(1.0, 2.0), Velocity(0.7, 0.8)));

// prepare a query that returns entities with the components required for the Update behavior
gen_tuple_query_component_providers!(
    Query,
    (&mut Position, &Velocity)
);

let dt = 0.1;
for (_, mut entity) in world.query_mut::<Query>() {
    // apply the Update behavior to the entity
    entity.update(dt);

    let position = entity.0;
    assert_eq!(position.0, 1.07);
    assert_eq!(position.1, 2.08);
}
```
