use {
    crate::*,
    static_assertions::assert_impl_all,
};



pub use hecs::{
    Iter, EntityRef, SpawnBatchIter, SpawnColumnBatchIter, ColumnBatch, NoSuchEntity, Bundle,
};



/// An unordered collection of entities, each having any number of distinctly typed components
///
/// Similar to `HashMap<Entity, Vec<Box<dyn Any>>>` where each `Vec` never contains two of the same
/// type, but far more efficient to traverse.
///
/// The components of entities who have the same set of component types are stored in contiguous
/// runs, allowing for extremely fast, cache-friendly iteration.
///
/// There is a maximum number of unique entity IDs, which means that there is a maximum number of live
/// entities. When old entities are despawned, their IDs will be reused on a future entity, and
/// old `Entity` values with that ID will be invalidated.
///
/// ### Collisions
///
/// If an entity is despawned and its `Entity` handle is preserved over the course of billions of
/// following spawns and despawns, that handle may, in rare circumstances, collide with a
/// newly-allocated `Entity` handle. Very long-lived applications should therefore limit the period
/// over which they may retain handles of despawned entities.
pub struct World {
    pub(crate) inner: hecs::World,
    /// A dummy entity to push resources on.
    pub resource_entity: Entity,
}
assert_impl_all!(World: Send, Sync, Default, std::fmt::Debug);

impl World {
    /// Creates an empty world.
    pub fn new() -> Self {
        let world = hecs::World::new();
        let resource_entity = world.reserve_entity();

        Self { inner: world, resource_entity }
    }

    /// Create an entity with certain components
    ///
    /// Returns the ID of the newly created entity.
    ///
    /// Arguments can be tuples, structs annotated with [`#[derive(Bundle)]`](macro@Bundle), or the
    /// result of calling [`build`](crate::EntityBuilder::build) on an
    /// [`EntityBuilder`](crate::EntityBuilder), which is useful if the set of components isn't
    /// statically known. To spawn an entity with only one component, use a one-element tuple like
    /// `(x,)`.
    ///
    /// Any type that satisfies `Send + Sync + 'static` can be used as a component.
    ///
    /// # Example
    /// ```
    /// # use terramine_ecs::*;
    /// let mut world = World::new();
    /// let a = world.spawn((123, "abc"));
    /// let b = world.spawn((456, true));
    /// ```
    pub fn spawn(&mut self, components: impl DynamicBundle) -> Entity {
        self.inner.spawn(components)
    }

    pub fn spawn_one(&mut self, component: impl Component) -> Entity {
        self.spawn((component,))
    }

    pub fn spawn_empty(&mut self) -> Entity {
        self.spawn(())
    }

    /// Create an entity with certain components and a specific [`Entity`] handle.
    ///
    /// See [`spawn`](Self::spawn).
    ///
    /// Despawns any existing entity with the same [`Entity::id`].
    ///
    /// Useful for easy handle-preserving deserialization. Be cautious resurrecting old `Entity`
    /// handles in already-populated worlds as it vastly increases the likelihood of collisions.
    ///
    /// # Example
    /// ```
    /// # use terramine_ecs::*;
    /// let mut world = World::new();
    /// let a = world.spawn((123, "abc"));
    /// let b = world.spawn((456, true));
    /// world.despawn(a);
    /// assert!(!world.contains(a));
    /// // all previous Entity values pointing to 'a' will be live again, instead pointing to the new entity.
    /// world.spawn_at(a, (789, "ABC"));
    /// assert!(world.contains(a));
    /// ```
    pub fn spawn_at(&mut self, handle: Entity, components: impl DynamicBundle) {
        self.inner.spawn_at(handle, components)
    }

    /// Efficiently spawn a large number of entities with the same statically-typed components
    ///
    /// Faster than calling [`spawn`](Self::spawn) repeatedly with the same components, but requires
    /// that component types are known at compile time.
    ///
    /// # Example
    /// ```
    /// # use terramine_ecs::*;
    /// let mut world = World::new();
    /// let entities = world.spawn_batch((0..1_000).map(|i| (i, "abc"))).collect::<Vec<_>>();
    /// for i in 0..1_000 {
    ///     assert_eq!(*world.get::<&i32>(entities[i]).unwrap(), i as i32);
    /// }
    /// ```
    pub fn spawn_batch<I>(&mut self, iter: I) -> SpawnBatchIter<'_, I::IntoIter>
    where
        I: IntoIterator,
        I::Item: Bundle + 'static,
    {
        self.inner.spawn_batch(iter)
    }

    /// Super-efficiently spawn the contents of a [`ColumnBatch`].
    ///
    /// The fastest, but most specialized, way to spawn large numbers of entities. Useful for high
    /// performance deserialization. Supports dynamic component types.
    pub fn spawn_column_batch(&mut self, batch: ColumnBatch) -> SpawnColumnBatchIter<'_> {
        self.inner.spawn_column_batch(batch)
    }

    /// Hybrid of [`spawn_column_batch`](Self::spawn_column_batch) and [`spawn_at`](Self::spawn_at).
    pub fn spawn_column_batch_at(&mut self, handles: &[Entity], batch: ColumnBatch) {
        self.inner.spawn_column_batch_at(handles, batch)
    }

    /// Destroy an entity and all its components.
    ///
    /// See also [`take`](Self::take).
    pub fn despawn(&mut self, entity: Entity) -> Result<(), NoSuchEntity> {
        self.inner.despawn(entity)
    }

    /// Ensure at least `additional` entities with exact components `T` can be spawned without reallocating.
    pub fn reserve<T: Bundle + 'static>(&mut self, additional: u32) {
        self.inner.reserve::<T>(additional)
    }

    /// Despawn all entities.
    ///
    /// Preserves allocated storage for reuse but clears metadata so that [`Entity`]
    /// values will repeat (in contrast to [`despawn`][Self::despawn]).
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Whether `entity` still exists.
    pub fn contains(&self, entity: Entity) -> bool {
        self.inner.contains(entity)
    }

    /// Efficiently iterate over all entities that have certain components, using dynamic borrow
    /// checking
    ///
    /// Prefer [`query_mut`](Self::query_mut) when concurrent access to the [`World`] is not required.
    ///
    /// Calling `iter` on the returned value yields `(Entity, Q)` tuples, where `Q` is some query
    /// type. A query type is any type for which an implementation of [`Query`] exists, e.g. `&T`,
    /// `&mut T`, a tuple of query types, or an `Option` wrapping a query type, where `T` is any
    /// component type. Components queried with `&mut` must only appear once. Entities which do not
    /// have a component type referenced outside of an `Option` will be skipped.
    ///
    /// Entities are yielded in arbitrary order.
    ///
    /// The returned [`QueryBorrow`] can be further transformed with combinator methods; see its
    /// documentation for details.
    ///
    /// Iterating a query will panic if it would violate an existing unique reference or construct
    /// an invalid unique reference. This occurs when two simultaneously-active queries could expose
    /// the same entity. Simultaneous queries can access the same component type if and only if the
    /// world contains no entities that have all components required by both queries, assuming no
    /// other component borrows are outstanding.
    ///
    /// Iterating a query yields references with lifetimes bound to the [`QueryBorrow`] returned
    /// here. To ensure those are invalidated, the return value of this method must be dropped for
    /// its dynamic borrows from the world to be released. Similarly, lifetime rules ensure that
    /// references obtained from a query cannot outlive the [`QueryBorrow`].
    ///
    /// # Example
    /// ```
    /// # use terramine_ecs::*;
    /// let mut world = World::new();
    /// let a = world.spawn((123, true, "abc"));
    /// let b = world.spawn((456, false));
    /// let c = world.spawn((42, "def"));
    /// let entities = world.query::<(&i32, &bool)>()
    ///     .iter()
    ///     .map(|(e, (&i, &b))| (e, i, b)) // Copy out of the world
    ///     .collect::<Vec<_>>();
    /// assert_eq!(entities.len(), 2);
    /// assert!(entities.contains(&(a, 123, true)));
    /// assert!(entities.contains(&(b, 456, false)));
    /// ```
    pub fn query<Q: Query>(&self) -> QueryBorrow<'_, Q> {
        self.inner.query::<Q>()
    }

    /// Query a uniquely borrowed world.
    ///
    /// Like [`query`](Self::query), but faster because dynamic borrow checks can be skipped. Note
    /// that, unlike [`query`](Self::query), this returns an `IntoIterator` which can be passed
    /// directly to a `for` loop.
    pub fn query_mut<Q: Query>(&mut self) -> QueryMut<'_, Q> {
        self.inner.query_mut::<Q>()
    }

    /// Prepare a query against a single entity, using dynamic borrow checking
    ///
    /// Prefer [`query_one_mut`](Self::query_one_mut) when concurrent access to the [`World`] is not
    /// required.
    ///
    /// Call [`get`](QueryOne::get) on the resulting [`QueryOne`] to actually execute the query. The
    /// [`QueryOne`] value is responsible for releasing the dynamically-checked borrow made by
    /// `get`, so it can't be dropped while references returned by `get` are live.
    ///
    /// Handy for accessing multiple components simultaneously.
    ///
    /// # Example
    /// ```
    /// # use terramine_ecs::*;
    /// let mut world = World::new();
    /// let a = world.spawn((123, true, "abc"));
    /// // The returned query must outlive the borrow made by `get`
    /// let mut query = world.query_one::<(&mut i32, &bool)>(a).unwrap();
    /// let (number, flag) = query.get().unwrap();
    /// if *flag { *number *= 2; }
    /// assert_eq!(*number, 246);
    /// ```
    pub fn query_one<Q: Query>(&self, entity: Entity) -> Result<QueryOne<'_, Q>, NoSuchEntity> {
        self.inner.query_one::<Q>(entity)
    }

    /// Query a single entity in a uniquely borrow world
    ///
    /// Like [`query_one`](Self::query_one), but faster because dynamic borrow checks can be
    /// skipped. Note that, unlike [`query_one`](Self::query_one), on success this returns the
    /// query's results directly.
    pub fn query_one_mut<Q: Query>(&mut self, entity: Entity) -> Result<Q::Item<'_>, QueryOneError> {
        self.inner.query_one_mut::<Q>(entity)
    }

    /// Short-hand for [`entity`](Self::entity) followed by [`EntityRef::get`]
    pub fn get<'a, T: ComponentRef<'a>>(&'a self, entity: Entity) -> Result<T::Ref, ComponentError> {
        self.inner.get::<T>(entity)
    }

    /// Short-hand for [`entity`](Self::entity) followed by [`EntityRef::satisfies`]
    pub fn satisfies<Q: Query>(&self, entity: Entity) -> Result<bool, NoSuchEntity> {
        self.inner.satisfies::<Q>(entity)
    }

    /// Access an entity regardless of its component types
    ///
    /// Does not immediately borrow any component.
    pub fn entity(&self, entity: Entity) -> Result<EntityRef<'_>, NoSuchEntity> {
        self.inner.entity(entity)
    }

    /// Access an entity regardless of its component types
    ///
    /// Does not immediately borrow any component.
    pub fn resource_entity(&self) -> EntityRef<'_> {
        self.entity(self.resource_entity)
            .expect("World::resource_entity shoud exist")
    }

    /// Access an entity regardless of its component types
    ///
    /// Does not immediately borrow any component.
    pub fn resource<'a, T: ComponentRef<'a>>(&'a self) -> Result<T::Ref, ComponentError> {
        self.get::<T>(self.resource_entity)
    }

    /// Access a resource with [`Copy`].
    pub fn copy_resource<T: Component + Copy>(&self) -> Result<T, ComponentError> {
        self.get::<&T>(self.resource_entity).map(|r| *r)
    }

    /// Given an id obtained from [`Entity::id`], reconstruct the still-live [`Entity`].
    ///
    /// # Safety
    ///
    /// `id` must correspond to a currently live [`Entity`]. A despawned or never-allocated `id`
    /// will produce undefined behavior.
    pub unsafe fn find_entity_from_id(&self, id: u32) -> Entity {
        self.inner.find_entity_from_id(id)
    }

    /// Iterate over all entities in the world
    ///
    /// Entities are yielded in arbitrary order. Prefer [`query`](Self::query) for better
    /// performance when components will be accessed in predictable patterns.
    ///
    /// # Example
    /// ```
    /// # use terramine_ecs::*;
    /// let mut world = World::new();
    /// let a = world.spawn(());
    /// let b = world.spawn(());
    /// let ids = world.iter().map(|entity_ref| entity_ref.entity()).collect::<Vec<_>>();
    /// assert_eq!(ids.len(), 2);
    /// assert!(ids.contains(&a));
    /// assert!(ids.contains(&b));
    /// ```
    pub fn iter(&self) -> Iter<'_> {
        self.inner.iter()
    }

    /// Add `components` to `entity`
    ///
    /// Computational cost is proportional to the number of components `entity` has. If an entity
    /// already has a component of a certain type, it is dropped and replaced.
    ///
    /// When inserting a single component, see [`insert_one`](Self::insert_one) for convenience.
    ///
    /// # Example
    /// ```
    /// # use terramine_ecs::*;
    /// let mut world = World::new();
    /// let e = world.spawn((123, "abc"));
    /// world.insert(e, (456, true));
    /// assert_eq!(*world.get::<&i32>(e).unwrap(), 456);
    /// assert_eq!(*world.get::<&bool>(e).unwrap(), true);
    /// ```
    pub fn insert(&mut self, entity: Entity, components: impl DynamicBundle) -> Result<(), NoSuchEntity> {
        self.inner.insert(entity, components)
    }

    /// Add `component` to `entity`
    ///
    /// See [`insert`](Self::insert).
    pub fn insert_one(&mut self, entity: Entity, component: impl Component) -> Result<(), NoSuchEntity> {
        self.inner.insert_one(entity, component)
    }

    /// Add `resource` to world.
    pub fn insert_resource(&mut self, resource: impl Component) {
        self.insert_one(self.resource_entity, resource)
            .expect("World::resource_entity exists by world creation");
    }

    /// Uses [`Default`] to initialize a `resource`.
    pub fn init_resource<T: Component + Default>(&mut self) {
        self.insert_resource(T::default())
    }

    /// Remove components from `entity`
    ///
    /// Computational cost is proportional to the number of components `entity` has. The entity
    /// itself is not removed, even if no components remain; use `despawn` for that. If any
    /// component in `T` is not present in `entity`, no components are removed and an error is
    /// returned.
    ///
    /// When removing a single component, see [`remove_one`](Self::remove_one) for convenience.
    ///
    /// # Example
    /// ```
    /// # use terramine_ecs::*;
    /// let mut world = World::new();
    /// let e = world.spawn((123, "abc", true));
    /// assert_eq!(world.remove::<(i32, &str)>(e), Ok((123, "abc")));
    /// assert!(world.get::<&i32>(e).is_err());
    /// assert!(world.get::<&&str>(e).is_err());
    /// assert_eq!(*world.get::<&bool>(e).unwrap(), true);
    /// ```
    pub fn remove<T: Bundle + 'static>(&mut self, entity: Entity) -> Result<T, ComponentError> {
        self.inner.remove::<T>(entity)
    }

    /// Remove the `T` component from `entity`
    ///
    /// See [`remove`](Self::remove).
    pub fn remove_one<T: Component>(&mut self, entity: Entity) -> Result<T, ComponentError> {
        self.inner.remove_one::<T>(entity)
    }

    /// Removes `resource` from `world`.
    pub fn remove_resource<T: Component>(&mut self) -> Result<T, ComponentError> {
        self.remove_one::<T>(self.resource_entity)
    }

    /// Remove `S` components from `entity` and then add `components`
    ///
    /// This has the same effect as calling [`remove::<S>`](Self::remove) and then [`insert::<T>`](Self::insert),
    /// but is more efficient as the intermediate archetype after removal but before insertion is skipped.
    pub fn exchange<S: Bundle + 'static, T: DynamicBundle>(
        &mut self, entity: Entity, components: T,
    ) -> Result<S, ComponentError> {
        self.inner.exchange::<S, T>(entity, components)
    }

    /// Remove the `S` component from `entity` and then add `component`
    ///
    /// See [`exchange`](Self::exchange).
    pub fn exchange_one<S: Component, T: Component>(
        &mut self, entity: Entity, component: T,
    ) -> Result<S, ComponentError> {
        self.inner.exchange_one(entity, component)
    }

    /// Borrow a single component of `entity` without safety checks
    ///
    /// `T` must be a shared or unique reference to a component type.
    ///
    /// Should only be used as a building block for safe abstractions.
    ///
    /// # Safety
    ///
    /// `entity` must have been previously obtained from this [`World`], and no unique borrow of the
    /// same component of `entity` may be live simultaneous to the returned reference.
    pub unsafe fn get_unchecked<'a, T: ComponentRef<'a>>(
        &'a self, entity: Entity,
    ) -> Result<T, ComponentError> {
        self.inner.get_unchecked(entity)
    }

    /// Borrow a resource without safety checks
    ///
    /// `T` must be a shared or unique reference to a component type.
    ///
    /// Should only be used as a building block for safe abstractions.
    ///
    /// # Safety
    ///
    /// `entity` must have been previously obtained from this [`World`], and no unique borrow of the
    /// same component of `entity` may be live simultaneous to the returned reference.
    pub unsafe fn get_resource_unchecked<'a, T: ComponentRef<'a>>(&'a self) -> Result<T, ComponentError> {
        self.get_unchecked(self.resource_entity)
    }

    /// Convert all reserved entities into empty entities that can be iterated and accessed
    ///
    /// Invoked implicitly by operations that add or remove components or entities, i.e. all
    /// variations of `spawn`, `despawn`, `insert`, and `remove`.
    pub fn flush(&mut self) {
        self.inner.flush()
    }

    /// Inspect the archetypes that entities are organized into
    ///
    /// Useful for dynamically scheduling concurrent queries by checking borrows in advance, and for
    /// efficient serialization.
    pub fn archetypes(&self) -> impl ExactSizeIterator<Item = &'_ Archetype> + '_ {
        self.inner.archetypes()
    }

    /// Despawn `entity`, yielding a [`DynamicBundle`] of its components
    ///
    /// Useful for moving entities between worlds.
    pub fn take(&mut self, entity: Entity) -> Result<TakenEntity<'_>, NoSuchEntity> {
        self.inner.take(entity)
    }

    /// Returns a distinct value after `archetypes` is changed
    ///
    /// Store the current value after deriving information from [`archetypes`](Self::archetypes),
    /// then check whether the value returned by this function differs before attempting an
    /// operation that relies on its correctness. Useful for determining whether e.g. a concurrent
    /// query execution plan is still correct.
    ///
    /// The generation may be, but is not necessarily, changed as a result of adding or removing any
    /// entity or component.
    ///
    /// # Example
    /// ```
    /// # use terramine_ecs::*;
    /// let mut world = World::new();
    /// let initial_gen = world.archetypes_generation();
    /// world.spawn((123, "abc"));
    /// assert_ne!(initial_gen, world.archetypes_generation());
    /// ```
    pub fn archetypes_generation(&self) -> ArchetypesGeneration {
        self.inner.archetypes_generation()
    }

    /// Number of currently live entities.
    pub fn len(&self) -> u32 {
        self.inner.len()
    }

    /// Whether no entities are live.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl std::fmt::Debug for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "World {{ ... }}")
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> IntoIterator for &'a World {
    type IntoIter = Iter<'a>;
    type Item = EntityRef<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

impl<A: DynamicBundle> Extend<A> for World {
    fn extend<T: IntoIterator<Item = A>>(&mut self, iter: T) {
        for x in iter {
            self.inner.spawn(x);
        }
    }
}

impl<A: DynamicBundle> core::iter::FromIterator<A> for World {
    fn from_iter<I: IntoIterator<Item = A>>(iter: I) -> Self {
        let mut world = World::new();
        world.extend(iter);
        world
    }
}



#[cfg(test)]
mod tests {
    #![allow(unused)]

    use super::*;

    #[derive(Default)]
    struct Timer(f32);

    #[test]
    fn test_resources() {
        let mut world = World::new();
        world.insert_resource(Timer(12.1));

        println!("{}", world.resource::<&Timer>().unwrap().0);

        {
            let mut timer = world.resource::<&mut Timer>().unwrap();
            timer.0 += 1.0;
        }

        println!("{}", world.resource::<&Timer>().unwrap().0);
    }

    #[test]
    fn test_borrows() {
        let mut world = World::new();
        world.init_resource::<Timer>();
        world.spawn((12.0_f32,));

        let mut timer = world.resource::<&mut Timer>().unwrap();
        for (entity, time) in world.query::<(&mut f32)>().into_iter() {
            *time += timer.0;
            timer.0 += 0.1;
        }
    }
}