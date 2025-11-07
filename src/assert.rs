#[macro_export]
macro_rules! assert_panics {
    ($code:expr) => {
        {
            print!("expecting panic:\n----------------------------------");
            let prev_backtrace = std::env::var("RUST_BACKTRACE").unwrap();
            unsafe { std::env::set_var("RUST_BACKTRACE", "0"); }
            assert!(
                std::panic::catch_unwind(|| {$code}).is_err(),
                "Expected panic, but no panic occurred"
            );
            unsafe { std::env::set_var("RUST_BACKTRACE", prev_backtrace); }
            println!("----------------------------------\ntask failed successfully");
        }
    };
}