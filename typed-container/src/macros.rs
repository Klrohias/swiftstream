#[macro_export]
macro_rules! ensure_services_available {
    ($container:expr, $type_name:ty) => {
        _ = $container.get::<$type_name>();
    };
    ($container:expr, $type_name:ty, $($type_names:ty),+) => {
        ensure_services_available!($container, $type_name);
        ensure_services_available!($container, $($type_names),+);
    };
}

#[cfg(test)]
mod tests {
    use crate::Container;

    #[derive(Clone)]
    struct A;

    #[derive(Clone)]
    struct B;

    #[test]
    fn ensure() {
        let c = Container::new();
        c.register_constructor(|_| A);
        c.register_constructor(|_| B);

        ensure_services_available!(c, A, B);
    }
}
