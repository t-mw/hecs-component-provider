//! Easily define behavior for sets of components when using the [hecs](https://github.com/Ralith/hecs/) ECS library.
//!
//! ```rust
//! use hecs_component_provider::{
//!     default_trait_impl, gen_tuple_query_component_providers,
//!     ComponentProvider, ComponentProviderMut
//! };
//!
//! struct Position(f32, f32);
//! struct Velocity(f32, f32);
//! struct Enemy { shot_count: i32 };
//!
//! // implement a behavior for all entities that provide the required components
//! #[default_trait_impl]
//! trait ApplyVelocity: ComponentProviderMut<Position> + ComponentProvider<Velocity> {
//!     fn apply_velocity(&mut self, dt: f32) {
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
//! // prepare a query that returns entities with the components required for the behavior
//! gen_tuple_query_component_providers!(
//!     MovableQuery,
//!     (&mut Position, &Velocity)
//! );
//!
//! let dt = 0.1;
//! # let mut found = false;
//! for (_, mut entity) in world.query_mut::<MovableQuery>() {
//!     // apply the behavior to the entity
//!     entity.apply_velocity(dt);
//!
//!     let position = entity.0;
//!     assert_eq!(position.0, 1.07);
//!     assert_eq!(position.1, 2.08);
//!     # found = true;
//! }
//! # assert!(found);
//!
//!
//! // behaviors can depend on one another
//! #[default_trait_impl]
//! trait EnemyBehaviors: ApplyVelocity + ComponentProviderMut<Enemy> {
//!     fn shoot_and_move(&mut self, dt: f32) {
//!         self.shoot();
//!         self.apply_velocity(dt);
//!     }
//!
//!     fn shoot(&mut self) {
//!         let enemy: &mut Enemy = self.get_mut();
//!         enemy.shot_count += 1;
//!     }
//! }
//!
//! world.spawn((Enemy { shot_count: 0 }, Position(2.0, 3.0), Velocity(-0.7, -0.8)));
//!
//! // queries can be prepared using structs instead of tuples
//! #[derive(hecs::Query, ComponentProvider)]
//! struct EnemyQuery<'a> {
//!     enemy: &'a mut Enemy,
//!     position: &'a mut Position,
//!     velocity: &'a Velocity,
//! }
//!
//! let dt = 0.1;
//! # let mut found = false;
//! for (_, mut entity) in world.query_mut::<EnemyQuery>() {
//!     // apply the behavior to the entity
//!     entity.shoot_and_move(dt);
//!
//!     assert_eq!(entity.enemy.shot_count, 1);
//!     assert_eq!(entity.position.0, 1.93);
//!     assert_eq!(entity.position.1, 2.92);
//!     # found = true;
//! }
//! # assert!(found);
//! ```

#[doc(hidden)]
pub use gensym::gensym;

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

/// Attach to a component struct to implement [`ComponentProvider`] and [`ComponentProviderMut`] for the struct
///
/// This allows behavior methods that require only a single component to be called on the struct
/// itself, even if the struct is not the direct result of a query.
///
/// ```
/// use hecs_component_provider::{default_trait_impl, ComponentProvider, SelfComponentProvider};
///
/// #[derive(SelfComponentProvider)]
/// struct Position(i32, i32);
///
/// #[default_trait_impl]
/// trait DistanceCalculator: ComponentProvider<Position> {
///     fn calculate_distance_squared_to(&self, other: &Position) -> i32 {
///         let position: &Position = self.get();
///         let dx = other.0 - position.0;
///         let dy = other.1 - position.1;
///         dx * dx + dy * dy
///     }
/// }
///
/// let position = Position(1, 2);
/// let other = Position(5, 8);
/// let distance_squared = position.calculate_distance_squared_to(&other);
/// assert_eq!(distance_squared, 52);
/// ```
pub use hecs_component_provider_macros::SelfComponentProvider;

/// Attach to a struct that derives [`hecs::Bundle`] or [`hecs::Query`] to generate component provider implementations for those structs.
///
/// ```
/// use hecs_component_provider::{
///     default_trait_impl, ComponentProvider, ComponentProviderMut
/// };
///
/// #[derive(Debug, Eq, PartialEq)]
/// struct Position(i32, i32);
/// #[derive(Debug, Eq, PartialEq)]
/// struct Velocity(i32, i32);
///
/// #[default_trait_impl]
/// trait ApplyVelocity: ComponentProviderMut<Position> + ComponentProvider<Velocity> {
///     fn apply_velocity(&mut self) {
///         // bind with let to disambiguate between component providers for Position and Velocity:
///         let &Velocity(vx, vy) = self.get();
///         // or use fully qualified syntax:
///         assert!(matches!(ComponentProvider::<Velocity>::get(self), &Velocity(_, _)));

///         let position: &mut Position = self.get_mut();
///         position.0 += vx;
///         position.1 += vy;
///     }
/// }
///
/// #[derive(hecs::Bundle, ComponentProvider)]
/// struct Entity {
///     position: Position,
///     velocity: Velocity,
/// }
///
/// #[derive(hecs::Query, ComponentProvider)]
/// struct MovableQuery<'a> {
///     position: &'a mut Position,
///     velocity: &'a Velocity,
/// }
///
/// let mut world = hecs::World::new();
/// let mut spawn_entity = Entity {
///     position: Position(10, 20),
///     velocity: Velocity(7, 8)
/// };
/// spawn_entity.apply_velocity(); // uses ComponentProvider implementation on Bundle
/// world.spawn(spawn_entity);
///
/// for (_, mut entity) in world.query_mut::<MovableQuery>() {
///     entity.apply_velocity(); // uses ComponentProvider implementation on Query
///     let position: &Position = entity.get();
///     assert_eq!(position, &Position(24, 36)); // apply_velocity has been applied twice by now
/// }
/// ```
pub use hecs_component_provider_macros::ComponentProvider;

/// Implement the attached trait for all types that implement the trait's supertraits
///
/// This can be used to reduce boilerplate when defining behavior traits. Attach the macro to a behavior
/// trait that requires certain components to exist and the trait will be implemented for all entities
/// that have those components.
///
/// ```
/// use hecs_component_provider::{default_trait_impl, ComponentProviderMut};
///
/// #[default_trait_impl]
/// trait MoveRight: ComponentProviderMut<Position> {
///    fn move_right(&mut self) {
///        let position: &mut Position = self.get_mut();
///        position.0 += 1;
///    }
/// }
/// // `default_trait_impl` generates the following, which would otherwise need to be manually provided:
/// // impl <T> move_right for T where T: ComponentProviderMut<Position> {}
///
/// # #[derive(hecs_component_provider::SelfComponentProvider)]
/// # struct Position(i32, i32);
/// # let mut position = Position(1, 2);
/// # position.move_right();
/// # assert_eq!(position.0, 2);
/// ```
pub use hecs_component_provider_macros::default_trait_impl;

/// Prepare a tuple query that includes component provider implementations for the returned entities
///
/// The first argument to the macro is the name of the query type that you would like to generate,
/// which can then be passed to the query methods on [`hecs::World`].
/// The second argument is the tuple of components that the query will return.
///
/// ```
/// use hecs_component_provider::{
///     gen_tuple_query_component_providers, ComponentProvider, ComponentProviderMut
/// };
///
/// #[derive(Debug, Eq, PartialEq)]
/// struct Position(i32, i32);
/// #[derive(Debug, Eq, PartialEq)]
/// struct Velocity(i32, i32);
///
/// let mut world = hecs::World::new();
/// world.spawn((Position(10, 20), Velocity(7, 8)));
///
/// gen_tuple_query_component_providers!(MovableQuery, (&mut Position, &Velocity));
///
/// for (_, mut entity) in world.query_mut::<MovableQuery>() {
///     assert_eq!(entity.get_mut(), &mut Position(10, 20));
///
///     // bind with let to disambiguate between component providers for Position and Velocity:
///     let _velocity: &Velocity = entity.get();
///     // or use fully qualified syntax:
///     assert_eq!(ComponentProvider::<Velocity>::get(&entity), &Velocity(7, 8));
/// }
/// ```
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
        #[derive(::hecs::Query, $crate::ComponentProvider)]
        struct $gensym<'a>($($tt)*);
        type $alias<'a> = $gensym<'a>;
    };

    // Begin with an empty stack.
    ($alias:ident, $($input:tt)+) => {
        gen_tuple_query_component_providers!($alias, @(()) $($input)*);
    };
}
