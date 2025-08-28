#[macro_export]
macro_rules! halextra {
    ($prefix:expr, $name:tt => $value: expr) => {
        $prefix.with_link($name, $value)
    };
    ($prefix:expr, $name:tt >> $value: expr) => {
        $prefix.with_link($name, $crate::HalLink::new($value).templated(true))
    };
    ($prefix:expr, $name:tt > $value: expr) => {
        $prefix.with_link($name, $crate::HalLink::new($value))
    };
    ($prefix:expr, $name:tt = $extra_value:expr) => {
        $prefix.with_extra_data($name, $extra_value)
    };
    ($prefix:expr, $name:tt >> $value:expr, $($key:tt $op:tt $val:expr),* ) => {
        $crate::halextra!($prefix.with_link($name, $crate::HalLink::new($value).templated(true)), $($key $op $val),*)
    };
    ($prefix:expr, $name:tt > $value:expr, $($key:tt $op:tt $val:expr),* ) => {
        $crate::halextra!($prefix.with_link($name, $crate::HalLink::new($value)), $($key $op $val),*)
    };
    ($prefix:expr, $name:tt => $value:expr, $($key:tt $op:tt $val:expr),* ) => {
        $crate::halextra!($prefix.with_link($name, $value), $($key $op $val),*)
    };
    ($prefix:expr, $name:tt = $extra_value:expr, $($key:tt $op:tt $val:expr),* ) => {
        $crate::halextra!($prefix.with_extra_data($name, $extra_value), $($key $op $val),*)
    };
}

#[macro_export]
macro_rules! hal {
    () => { HalResource::new(()) };
    ($payload:expr ) => { $crate::HalResource::new($payload) };
    ($payload:expr, $($key:tt $op:tt $val:expr),*) => {
        $crate::halextra!($crate::HalResource::new($payload), $($key $op $val),*)
    };
}


#[macro_export]
 macro_rules! hal_extend {
    ($initial:expr) => { $initial };
    ($initial:expr, $($key:tt $op:tt $val:expr),*) => {
        $crate::halextra!($initial, $($key $op $val),*)
    };
}