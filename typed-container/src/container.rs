use std::{
    any::{Any, TypeId, type_name},
    collections::HashMap,
    sync::{Arc, RwLock},
};

use log::warn;
use smol_str::SmolStr;

use crate::{GetServiceError, RegisterError};

#[derive(Clone)]
pub struct Container(Arc<RwLock<ContainerImpl>>);

impl Container {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(ContainerImpl::default())))
    }

    fn register_internal(&self, boxed_service: BoxedService) -> Result<(), RegisterError> {
        {
            let read = self.0.read()?;

            if read.services.contains_key(&boxed_service.type_id) {
                return Err(RegisterError::Duplicated(boxed_service.type_name.into()));
            }
        }

        {
            let mut write = self.0.write()?;
            write
                .services
                .insert(boxed_service.type_id.clone(), boxed_service);
        }

        Ok(())
    }

    fn register_constructor_internal(
        &self,
        boxed_constructor: BoxedConstructor,
    ) -> Result<(), RegisterError> {
        {
            let read = self.0.read()?;

            if read.constructors.contains_key(&boxed_constructor.type_id) {
                return Err(RegisterError::Duplicated(
                    boxed_constructor.type_name.into(),
                ));
            }
        }

        {
            let mut write = self.0.write()?;
            write
                .constructors
                .insert(boxed_constructor.type_id, Arc::new(boxed_constructor));
        }

        Ok(())
    }

    pub fn register<T: Clone + 'static>(&self, value: T) {
        if let Err(e) = self.register_internal(BoxedService::from(value)) {
            warn!("Failed to register service: {}", e);
        }
    }

    pub fn register_constructor<T: Clone + 'static>(
        &self,
        value: impl Fn(Container) -> T + 'static,
    ) {
        if let Err(e) = self.register_constructor_internal(BoxedConstructor::from(value)) {
            warn!("Failed to register constructor: {}", e);
        }
    }

    pub fn get<T: Clone + 'static>(&self) -> T {
        self.try_get().unwrap()
    }

    pub fn try_get<T: Clone + 'static>(&self) -> Result<T, GetServiceError> {
        let type_id = TypeId::of::<T>();
        {
            let read = self.0.read()?;
            if let Some(s) = read.services.get(&type_id) {
                return match s.get_cloned() {
                    Some(v) => Ok(v),
                    None => Err(GetServiceError::FailDowncast(type_name::<T>().into())),
                };
            }

            if !read.constructors.contains_key(&type_id) {
                return Err(GetServiceError::NotFound(type_name::<T>().into()));
            }
        }

        let constructor = {
            let read = self.0.read()?;
            read.constructors.get(&type_id).unwrap().clone()
        };

        let new_value = constructor
            .construct::<T>(self.clone())
            .ok_or(GetServiceError::FailConstruct(type_name::<T>().into()))?;

        self.register_internal(BoxedService::from(new_value.clone()))?;
        Ok(new_value)
    }
}

#[derive(Default)]
struct ContainerImpl {
    pub constructors: HashMap<TypeId, Arc<BoxedConstructor>>,
    pub services: HashMap<TypeId, BoxedService>,
}

struct BoxedService {
    pub type_id: TypeId,
    pub type_name: SmolStr,
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
            type_name: type_name::<T>().into(),
            value: Box::new(value),
        }
    }
}

struct BoxedConstructor {
    pub type_id: TypeId,
    pub type_name: SmolStr,
    pub value: Box<dyn Fn(Container) -> Box<dyn Any>>,
}

impl BoxedConstructor {
    fn construct<T: Clone + 'static>(&self, container: Container) -> Option<T> {
        let value = (self.value)(container).downcast::<T>();
        match value {
            Err(_) => None,
            Ok(v) => Some(*v),
        }
    }
}

impl<T: Clone + 'static, F: Fn(Container) -> T + 'static> From<F> for BoxedConstructor {
    fn from(value: F) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>().into(),
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
        c.register("A".to_string());
        c.register(123 as u64);

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
}
