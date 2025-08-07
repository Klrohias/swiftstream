use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{Error, ErrorKind};

#[derive(Clone)]
pub struct Container<'a>(Arc<RwLock<ContainerImpl<'a>>>);

impl<'a> Container<'a> {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(ContainerImpl::default())))
    }

    fn register_service_internal(&self, boxed_service: BoxedService) -> Result<(), ErrorKind> {
        {
            let read = self.0.read().map_err(|_| ErrorKind::LockPoisoned)?;

            if read.services.contains_key(&boxed_service.type_id) {
                return Err(ErrorKind::Duplicated);
            }
        }

        {
            let mut write = self.0.write().map_err(|_| ErrorKind::LockPoisoned)?;
            write
                .services
                .insert(boxed_service.type_id.clone(), boxed_service);
        }

        Ok(())
    }

    fn register_constructor_internal(
        &self,
        boxed_constructor: BoxedConstructor<'a>,
    ) -> Result<(), ErrorKind> {
        {
            let read = self.0.read().map_err(|_| ErrorKind::LockPoisoned)?;

            if read.constructors.contains_key(&boxed_constructor.type_id) {
                return Err(ErrorKind::Duplicated);
            }
        }

        {
            let mut write = self.0.write().map_err(|_| ErrorKind::LockPoisoned)?;
            write
                .constructors
                .insert(boxed_constructor.type_id, Arc::new(boxed_constructor));
        }

        Ok(())
    }

    pub fn construct<T: Clone + 'static>(&self) -> T {
        self.try_construct().unwrap()
    }

    pub fn try_construct<T: Clone + 'static>(&self) -> Result<T, Error<T>> {
        let type_id = TypeId::of::<T>();
        let construct = {
            let read = self.0.read()?;

            read.constructors
                .get(&type_id)
                .ok_or(ErrorKind::NotFound)?
                .clone()
        };

        match construct.construct::<T>(self.clone()) {
            None => Err(ErrorKind::FailDowncast.into()),
            Some(v) => Ok(v),
        }
    }

    pub fn register_service<T: Clone + 'static>(&self, value: T) {
        self.register_service_internal(BoxedService::from(value))
            .unwrap()
    }

    pub fn try_register_service<T: Clone + 'static>(&self, value: T) -> Result<(), Error<T>> {
        Ok(self.register_service_internal(BoxedService::from(value))?)
    }

    pub fn register_constructor<T: Clone + 'static>(&self, value: impl Fn(Container) -> T + 'a) {
        self.register_constructor_internal(BoxedConstructor::from(value))
            .unwrap()
    }

    pub fn try_register_constructor<T: Clone + 'static>(
        &self,
        value: impl Fn(Container) -> T + 'a,
    ) -> Result<(), Error<T>> {
        Ok(self.register_constructor_internal(BoxedConstructor::from(value))?)
    }

    pub fn get<T: Clone + 'static>(&self) -> T {
        self.try_get().unwrap()
    }

    pub fn try_get<T: Clone + 'static>(&self) -> Result<T, Error<T>> {
        let type_id = TypeId::of::<T>();
        {
            let read = self.0.read()?;
            if let Some(s) = read.services.get(&type_id) {
                return match s.get_cloned() {
                    Some(v) => Ok(v),
                    None => Err(ErrorKind::FailDowncast.into()),
                };
            }

            if !read.constructors.contains_key(&type_id) {
                return Err(ErrorKind::NotFound.into());
            }
        }

        let new_value = self.try_construct::<T>()?;

        self.register_service_internal(BoxedService::from(new_value.clone()))?;
        Ok(new_value)
    }
}

#[derive(Default)]
struct ContainerImpl<'a> {
    pub constructors: HashMap<TypeId, Arc<BoxedConstructor<'a>>>,
    pub services: HashMap<TypeId, BoxedService>,
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
}
