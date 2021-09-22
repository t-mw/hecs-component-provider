//! Easily define behavior for sets of components when using the hecs ECS library.
//!
//! ```
//! use hecs_component_provider::{
//!     default_trait_impl, gen_tuple_query_component_providers,
//!     ComponentProvider, ComponentProviderMut
//! };
//!
//! struct Position(f32, f32);
//! struct Velocity(f32, f32);
//!
//! // implement a behavior for all entities that provide the required components
//! #[default_trait_impl]
//! trait Update: ComponentProviderMut<Position> + ComponentProvider<Velocity> {
//!     fn update(&mut self, dt: f32) {
//!         let &Velocity(vx, vy) = self.get();
//!         let position: &mut Position = self.get_mut();
//!         position.0 += vx * dt;
//!         position.1 += vy * dt;
//!     }
//! }
//!
//! let mut world = hecs::World::new();
//! world.spawn((Position(1.0, 2.0), Velocity(0.7, 0.8)));
//!
//! // prepare a query that returns entities with the components required for the Update behavior
//! gen_tuple_query_component_providers!(
//!     Query,
//!     (&mut Position, &Velocity)
//! );
//!
//! let dt = 0.1;
//! for (_, mut entity) in world.query_mut::<Query>() {
//!     // apply the Update behavior to the entity
//!     entity.update(dt);
//!
//!     let position = entity.0;
//!     assert_eq!(position.0, 1.07);
//!     assert_eq!(position.1, 2.08);
//! }
//! ```

pub use gensym::gensym;

pub use hecs_component_provider_macros::*;

pub trait ComponentProvider<Component> {
    fn get(&self) -> &Component;
}

pub trait ComponentProviderMut<Component>: ComponentProvider<Component> {
    fn get_mut(&mut self) -> &mut Component;
}

pub trait ComponentProviderOptional<Component> {
    fn get_optional(&self) -> Option<&Component>;
}

pub trait ComponentProviderOptionalMut<Component>: ComponentProviderOptional<Component> {
    fn get_optional_mut(&mut self) -> Option<&mut Component>;
}

/// Implement ComponentProvider for a tuple query type.
#[macro_export]
macro_rules! gen_tuple_query_component_providers {
    // Uses a TT muncher to add lifetimes: https://users.rust-lang.org/t/macro-to-replace-type-parameters/17903/2

    // Open parenthesis.
    ($alias:ident, @($($stack:tt)*) ($($first:tt)*) $($rest:tt)*) => {
        gen_tuple_query_component_providers!($alias, @(() $($stack)*) $($first)* __paren $($rest)*);
    };

    // Close parenthesis.
    ($alias:ident, @(($($close:tt)*) ($($top:tt)*) $($stack:tt)*) __paren $($rest:tt)*) => {
        gen_tuple_query_component_providers!($alias, @(($($top)* ($($close)*)) $($stack)*) $($rest)*);
    };

    // Replace `&` token with `& 'a`.
    ($alias:ident, @(($($top:tt)*) $($stack:tt)*) & $($rest:tt)*) => {
        gen_tuple_query_component_providers!($alias, @(($($top)* &'a) $($stack)*) $($rest)*);
    };

    // Replace `&&` token with `& 'a & 'a`.
    ($alias:ident, @(($($top:tt)*) $($stack:tt)*) && $($rest:tt)*) => {
        gen_tuple_query_component_providers!($alias, @(($($top)* &'a &'a) $($stack)*) $($rest)*);
    };

    // Munch a token that is not `&`.
    ($alias:ident, @(($($top:tt)*) $($stack:tt)*) $first:tt $($rest:tt)*) => {
        gen_tuple_query_component_providers!($alias, @(($($top)* $first) $($stack)*) $($rest)*);
    };

    // Done.
    ($alias:ident, @(($($top:tt)+))) => {
        $crate::gensym! { gen_tuple_query_component_providers!(impl $alias, $($top)+) }
    };

    ($gensym:ident, impl $alias:ident, ($($tt:tt)*)) => {
        #[derive(::hecs::Query, $crate::QueryComponentProvider)]
        struct $gensym<'a>($($tt)*);
        type $alias<'a> = $gensym<'a>;
    };

    // Begin with an empty stack.
    ($alias:ident, $($input:tt)+) => {
        gen_tuple_query_component_providers!($alias, @(()) $($input)*);
    };
}
