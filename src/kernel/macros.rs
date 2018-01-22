// -*- mode: rust; -*-

macro_rules! println {
    () => {{ print!("\n") }};
    ($fmt:expr) => {{ print!(concat!($fmt, "\n")) }};
    ($fmt:expr, $($arg:tt)*) => {{ print!(concat!($fmt, "\n"), $($arg)*) }};
}

#[allow_internal_unstable]
macro_rules! print {
    ($($arg:tt)*) => {{
        $crate::kernel::vga::print(format_args!($($arg)*));
    }}
}



macro_rules! once {
    ($($arg:tt)+) => {{
        fn __once__() {
            use $crate::core::sync::atomic::{ATOMIC_BOOL_INIT, AtomicBool, Ordering};
            static C : AtomicBool = ATOMIC_BOOL_INIT;
            assert!(!C.swap(true, Ordering::SeqCst), $($arg)+);
        } __once__();
    }}
}



macro_rules! const_assert {
    ($($condition:expr),+ $(,)*) => {{
        let _ = [(); 0 - !($($condition)&&+) as usize];
    }};
    ($label:ident; $($rest:tt)+) => {{
        #[allow(dead_code, non_snake_case)]
        fn $label() { const_assert!($($rest)+); }
    }};
}

macro_rules! assert_eq_size {
    ($x:ty, $($xs:ty),+ $(,)*) => {{
        $(let _ = transmute::<$x, $xs>;)+
    }};
    ($label:ident; $($rest:tt)+) => {{
        #[allow(dead_code, non_snake_case)]
        fn $label() { assert_eq_size!($($rest)+); }
    }};
}



macro_rules! int {
    ($arg:expr) => {{ asm!("int $0" :: "N" ($arg)); }}
}


macro_rules! count_tts {
    ($($tts:tt)*) => {<[()]>::len(&[$(replace_expr!($tts ())),*])};
}
