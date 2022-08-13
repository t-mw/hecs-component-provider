use hecs::World;

#[test]
fn self_component_provider_test() {
    use hecs_component_provider::{default_trait_impl, ComponentProvider, SelfComponentProvider};

    #[derive(SelfComponentProvider)]
    struct MyComponent(i32);

    #[default_trait_impl]
    trait ReturnInner: ComponentProvider<MyComponent> {
        fn return_inner(&self) -> i32 {
            let c: &MyComponent = self.get();
            c.0
        }
    }

    let c1 = MyComponent(123);
    assert_eq!(c1.return_inner(), 123);
}

#[test]
fn query_component_provider_basic_test() {
    use hecs_component_provider::ComponentProvider;

    #[derive(Debug)]
    struct MyComponent(i32);

    #[derive(hecs::Query, ComponentProvider)]
    struct MyQuery<'a> {
        integer: &'a i32,
        component: &'a MyComponent,
    }

    let mut world = World::new();
    let id = world.spawn((123, MyComponent(456), "abc"));
    let mut query = world
        .query_one::<MyQuery>(id)
        .expect("Entity should be returned");
    let entity = query.get().unwrap();

    let integer: &i32 = entity.get();
    assert_eq!(*integer, 123);

    let component: &MyComponent = entity.get();
    assert_eq!(component.0, 456);
}

#[test]
fn query_component_provider_complex_test() {
    use hecs_component_provider::{
        ComponentProvider, ComponentProviderMut, ComponentProviderOptional,
        ComponentProviderOptionalMut,
    };

    #[derive(Debug, Eq, PartialEq)]
    struct MyComponent(i32);

    #[derive(hecs::Query, ComponentProvider)]
    struct MyQuery<'a> {
        integer: &'a mut i32,
        boolean: Option<&'a bool>,
        component: Option<&'a mut MyComponent>,
        string: &'a &'a str,
    }

    let mut world = World::new();
    world.spawn((123, true, MyComponent(456), "abc"));
    world.spawn((42, false, MyComponent(789)));

    let mut query = world.query::<MyQuery>();
    let mut query_iter = query.iter();
    let (_, mut entity) = query_iter
        .next()
        .expect("At least one entity should be returned");
    assert!(
        query_iter.next().is_none(),
        "Only one entity should be returned"
    );

    let integer: &i32 = entity.get();
    assert_eq!(*integer, 123);

    let integer: &mut i32 = entity.get_mut();
    assert_eq!(*integer, 123);

    let boolean: Option<&bool> = entity.get_optional();
    assert_eq!(boolean, Some(&true));

    let component: Option<&MyComponent> = entity.get_optional();
    assert_eq!(component, Some(&MyComponent(456)));

    let component: Option<&mut MyComponent> = entity.get_optional_mut();
    assert_eq!(component, Some(&mut MyComponent(456)));

    let string: &&str = entity.get();
    assert_eq!(*string, "abc");
}

#[test]
fn gen_tuple_query_component_providers_test() {
    use hecs_component_provider::{
        gen_tuple_query_component_providers, ComponentProvider, ComponentProviderMut,
        ComponentProviderOptional, ComponentProviderOptionalMut,
    };

    #[derive(Debug, Eq, PartialEq)]
    struct MyComponent(i32);

    gen_tuple_query_component_providers!(
        MyQuery,
        (&mut i32, Option<&bool>, Option<&mut MyComponent>, &&str)
    );

    let mut world = World::new();
    world.spawn((123, true, MyComponent(456), "abc"));
    world.spawn((42, false, MyComponent(789)));

    let mut query = world.query::<MyQuery>();
    let mut query_iter = query.iter();
    let (_, mut entity) = query_iter
        .next()
        .expect("At least one entity should be returned");
    assert!(
        query_iter.next().is_none(),
        "Only one entity should be returned"
    );

    let integer: &i32 = entity.get();
    assert_eq!(*integer, 123);

    let integer: &mut i32 = entity.get_mut();
    assert_eq!(*integer, 123);

    let boolean: Option<&bool> = entity.get_optional();
    assert_eq!(boolean, Some(&true));

    let component: Option<&MyComponent> = entity.get_optional();
    assert_eq!(component, Some(&MyComponent(456)));

    let component: Option<&mut MyComponent> = entity.get_optional_mut();
    assert_eq!(component, Some(&mut MyComponent(456)));

    let string: &&str = entity.get();
    assert_eq!(*string, "abc");
}
