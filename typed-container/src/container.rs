use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

use crate::{Error, ErrorKind};

/// A container that you can register your services or constructors and handle their dependencies
#[derive(Clone)]
pub struct Container<'a>(Arc<RwLock<ContainerImpl<'a>>>);

impl<'a> Container<'a> {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(ContainerImpl::default())))
    }

    fn register_service_internal(&self, boxed_service: BoxedService) -> Result<(), ErrorKind> {
        let mut impl_obj = self.0.write()?;
        if impl_obj.services.contains_key(&boxed_service.type_id) {
            return Err(ErrorKind::Duplicated);
        }
        impl_obj
            .services
            .insert(boxed_service.type_id.clone(), boxed_service);

        Ok(())
    }

    fn register_constructor_internal(
        &self,
        boxed_constructor: BoxedConstructor<'a>,
    ) -> Result<(), ErrorKind> {
        let mut write = self.0.write()?;
        write
            .constructors
            .insert(boxed_constructor.type_id, Arc::new(boxed_constructor));

        Ok(())
    }

    fn construct_internal<T: Clone + 'static>(&self) -> Result<T, ErrorKind> {
        let type_id = TypeId::of::<T>();

        // check pending and get the constructor
        let constructor = {
            let impl_ref = self.0.read()?;

            if impl_ref.pending_construction.contains(&type_id) {
                return Err(ErrorKind::CircularReference);
            }

            impl_ref
                .constructors
                .get(&type_id)
                .ok_or(ErrorKind::NotFound)?
                .clone()
        };

        // add current type into pending
        self.0.write()?.pending_construction.insert(type_id.clone());

        // construct the object
        let construction = constructor.construct::<T>(self.clone());

        // remove pending
        self.0.write()?.pending_construction.remove(&type_id);

        match construction {
            None => Err(ErrorKind::FailDowncast),
            Some(v) => Ok(v),
        }
    }

    /// Register a new service.
    ///
    /// Panic if error occurred.
    pub fn register_service<T: Clone + 'static>(&self, value: T) {
        self.register_service_internal(BoxedService::from(value))
            .unwrap()
    }

    /// Register a new service.
    pub fn try_register_service<T: Clone + 'static>(&self, value: T) -> Result<(), Error<T>> {
        Ok(self.register_service_internal(BoxedService::from(value))?)
    }

    /// Register a new constructor or replace the old constructor.
    ///
    /// Panic if error occurred.
    pub fn register_constructor<T: Clone + 'static>(&self, value: impl Fn(Container) -> T + 'a) {
        self.register_constructor_internal(BoxedConstructor::from(value))
            .unwrap()
    }

    /// Register a new constructor or replace the old constructor.
    pub fn try_register_constructor<T: Clone + 'static>(
        &self,
        value: impl Fn(Container) -> T + 'a,
    ) -> Result<(), Error<T>> {
        Ok(self.register_constructor_internal(BoxedConstructor::from(value))?)
    }

    /// Get a service from the container, if the service doesn't exist but a available constructor exists, it will try to construct it.
    ///
    /// Panic if error occurred.
    pub fn get<T: Clone + 'static>(&self) -> T {
        self.try_get().unwrap()
    }

    /// Get a service from the container, if the service doesn't exist but a available constructor exists, it will try to construct it.
    pub fn try_get<T: Clone + 'static>(&self) -> Result<T, Error<T>> {
        let type_id = TypeId::of::<T>();
        {
            let impl_obj = self.0.read()?;

            if let Some(s) = impl_obj.services.get(&type_id) {
                return match s.get_cloned() {
                    Some(v) => Ok(v),
                    None => Err(ErrorKind::FailDowncast.into()),
                };
            }

            if !impl_obj.constructors.contains_key(&type_id) {
                return Err(ErrorKind::NotFound.into());
            }
        }

        let new_value = self.try_construct::<T>()?;

        self.register_service_internal(BoxedService::from(new_value.clone()))?;
        Ok(new_value)
    }

    /// Just construct a new object, whether it exists or not, it will be constructed and won't be inserted into container
    ///
    /// Panic if error occurred.
    pub fn construct<T: Clone + 'static>(&self) -> T {
        self.construct_internal().unwrap()
    }

    /// Just construct a new object, whether it exists or not, it will be constructed and won't be inserted into container
    pub fn try_construct<T: Clone + 'static>(&self) -> Result<T, Error<T>> {
        Ok(self.construct_internal()?)
    }

    /// Remove a service
    pub fn remove_service<T: Clone + 'static>(&self) -> Result<T, Error<T>> {
        Ok(self
            .0
            .write()?
            .services
            .remove(&TypeId::of::<T>())
            .ok_or(ErrorKind::NotFound)?
            .get_cloned::<T>()
            .ok_or(ErrorKind::FailDowncast)?)
    }

    /// Remove a constructor
    pub fn remove_constructor<T: Clone + 'static>(&self) -> Result<(), Error<T>> {
        self.0.write()?.constructors.remove(&TypeId::of::<T>());
        Ok(())
    }

    /// Remove all constructors and convert the lifetime to static
    pub fn into_static(self) -> Container<'static> {
        let mut services = HashMap::new();
        std::mem::swap(&mut self.0.write().unwrap().services, &mut services);

        Container::<'static>(Arc::new(RwLock::new(ContainerImpl {
            services,
            ..Default::default()
        })))
    }
}

#[derive(Default)]
struct ContainerImpl<'a> {
    pub constructors: HashMap<TypeId, Arc<BoxedConstructor<'a>>>,
    pub services: HashMap<TypeId, BoxedService>,
    pub pending_construction: HashSet<TypeId>,
}

struct BoxedService {
    pub type_id: TypeId,
    pub value: Box<dyn Any>,
}

impl BoxedService {
    fn get_cloned<T: Clone + 'static>(&self) -> Option<T> {
        self.value.downcast_ref::<T>().cloned()
    }
}

impl<T: Clone + 'static> From<T> for BoxedService {
    fn from(value: T) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            value: Box::new(value),
        }
    }
}

struct BoxedConstructor<'a> {
    pub type_id: TypeId,
    pub value: Box<dyn Fn(Container) -> Box<dyn Any> + 'a>,
}

impl<'a> BoxedConstructor<'a> {
    fn construct<T: Clone + 'static>(&self, container: Container) -> Option<T> {
        let value = (self.value)(container).downcast::<T>();
        match value {
            Err(_) => None,
            Ok(v) => Some(*v),
        }
    }
}

impl<'a, T: Clone + 'static, F: Fn(Container) -> T + 'a> From<F> for BoxedConstructor<'a> {
    fn from(value: F) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            value: Box::new(move |c| Box::new(value(c))),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::Container;

    #[test]
    fn basic_register() {
        let c = Container::new();
        c.register_service("A".to_string());
        c.register_service(123 as u64);

        assert_eq!(c.get::<String>(), "A");
        assert_eq!(c.get::<u64>(), 123);
    }

    #[test]
    fn basic_constructor() {
        let c = Container::new();
        c.register_constructor(|_| "A".to_string());
        c.register_constructor(|_| 123 as u64);

        assert_eq!(c.get::<String>(), "A");
        assert_eq!(c.get::<u64>(), 123);
    }

    #[allow(dead_code)]
    struct A {
        b: Arc<B>,
        d: Arc<D>,
    }

    #[allow(dead_code)]
    struct B {
        c: Arc<C>,
    }

    struct C;
    #[derive(Clone)]
    struct D;

    #[test]
    fn complex() {
        let c = Container::new();
        c.register_constructor(|container| {
            Arc::new(A {
                b: container.get(),
                d: container.get(),
            })
        });
        c.register_constructor(|container| Arc::new(B { c: container.get() }));
        c.register_constructor(|_| Arc::new(C));
        c.register_constructor(|_| Arc::new(D));

        _ = c.get::<Arc<A>>();
    }

    #[test]
    fn constructor_with_lifetime() {
        let outside_string = "A".to_string();
        let outside_d = D;

        let c = Container::new();
        c.register_constructor(|_| outside_string.clone());
        c.register_constructor(|_| outside_d.clone());

        assert_eq!(c.get::<String>(), "A");
    }

    #[derive(Debug)]
    struct RefA {
        pub _b: Arc<RefB>,
    }

    #[derive(Debug)]
    struct RefB {
        pub _a: Arc<RefA>,
    }

    #[test]
    #[should_panic]
    fn circular_reference() {
        let c = Container::new();
        c.register_constructor(|c| Arc::new(RefA { _b: c.get() }));
        c.register_constructor(|c| Arc::new(RefB { _a: c.get() }));

        _ = c.get::<Arc<RefA>>();
    }
}
