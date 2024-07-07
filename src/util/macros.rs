#[macro_export]
#[doc = "Create a HashMap with the given key and value types."]
macro_rules! map {
    ($key:ty, $val:ty) => {{
        let map: HashMap<$key, $val> = HashMap::new();
        map
    }};

    ($($key:expr => $val:expr), *) => {
        {
            let map = std::collections::HashMap::new();
            $( map.insert($key, $val); )*
            map
        }
    };
}

#[macro_export]
#[doc = "shorthand to convert str to String"]
macro_rules! s {
    ($value:tt) => {{
        $value.to_string()
    }};
}

#[macro_export]
#[doc = "panic and exit"]
macro_rules! exit {
    ($value:tt) => {{
        println!($value);
        std::process::exit(1)
    }};
}
