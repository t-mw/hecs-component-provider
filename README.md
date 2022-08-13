# hecs-component-provider

[![Crates.io](https://img.shields.io/crates/v/hecs-component-provider.svg)](https://crates.io/crates/hecs-component-provider)
[![Docs Status](https://docs.rs/hecs-component-provider/badge.svg)](https://docs.rs/hecs-component-provider)

Easily define behavior for sets of components when using the [hecs](https://github.com/Ralith/hecs/) ECS library.

```rust
use hecs_component_provider::{
    default_trait_impl, gen_tuple_query_component_providers,
    ComponentProvider, ComponentProviderMut
};

struct Position(f32, f32);
struct Velocity(f32, f32);
struct Enemy { shot_count: i32 };

// implement a behavior for all entities that provide the required components
#[default_trait_impl]
trait ApplyVelocity: ComponentProviderMut<Position> + ComponentProvider<Velocity> {
    fn apply_velocity(&mut self, dt: f32) {
        let &Velocity(vx, vy) = self.get();
        let position: &mut Position = self.get_mut();
        position.0 += vx * dt;
        position.1 += vy * dt;
    }
}

let mut world = hecs::World::new();
world.spawn((Position(1.0, 2.0), Velocity(0.7, 0.8)));

// prepare a query that returns entities with the components required for the behavior
gen_tuple_query_component_providers!(
    MovableQuery,
    (&mut Position, &Velocity)
);

let dt = 0.1;
for (_, mut entity) in world.query_mut::<MovableQuery>() {
    // apply the behavior to the entity
    entity.apply_velocity(dt);

    let position = entity.0;
    assert_eq!(position.0, 1.07);
    assert_eq!(position.1, 2.08);
}


// behaviors can depend on one another
#[default_trait_impl]
trait EnemyBehaviors: ApplyVelocity + ComponentProviderMut<Enemy> {
    fn shoot_and_move(&mut self, dt: f32) {
        self.shoot();
        self.apply_velocity(dt);
    }

    fn shoot(&mut self) {
        let enemy: &mut Enemy = self.get_mut();
        enemy.shot_count += 1;
    }
}

world.spawn((Enemy { shot_count: 0 }, Position(2.0, 3.0), Velocity(-0.7, -0.8)));

// queries can be prepared using structs instead of tuples
#[derive(hecs::Query, ComponentProvider)]
struct EnemyQuery<'a> {
    enemy: &'a mut Enemy,
    position: &'a mut Position,
    velocity: &'a Velocity,
}

let dt = 0.1;
for (_, mut entity) in world.query_mut::<EnemyQuery>() {
    // apply the behavior to the entity
    entity.shoot_and_move(dt);

    assert_eq!(entity.enemy.shot_count, 1);
    assert_eq!(entity.position.0, 1.93);
    assert_eq!(entity.position.1, 2.92);
}
```
