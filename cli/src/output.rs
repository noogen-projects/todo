#[macro_export]
macro_rules! out {
    ($($arg:tt)*) => {{
        print!($($arg)*)
    }};
}

#[macro_export]
macro_rules! outln {
    ($($arg:tt)*) => {{
        println!($($arg)*)
    }};
}

#[macro_export]
macro_rules! eout {
    ($($arg:tt)*) => {{
        eprint!($($arg)*)
    }};
}

#[macro_export]
macro_rules! eoutln {
    ($($arg:tt)*) => {{
        eprintln!($($arg)*)
    }};
}
