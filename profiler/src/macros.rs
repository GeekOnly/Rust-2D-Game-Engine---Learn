#[macro_export]
macro_rules! profile_scope {
    ($name:expr) => {
        #[cfg(feature = "enable_profiling")]
        let _timer = $crate::ScopeTimer::new($name);
    };
}

#[macro_export]
macro_rules! profile_function {
    () => {
        let func_name = {
            fn f() {}
            fn type_name_of<T>(_: T) -> &'static str {
                std::any::type_name::<T>()
            }
            type_name_of(f)
        };
        $crate::profile_scope!(func_name);
    };
}
