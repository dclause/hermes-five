extern crate hermes_macros;

use hermes_five::errors::Error;
#[allow(unused_imports)]
use hermes_five::utils::tokio;

#[hermes_macros::runtime]
async fn example_runtime_f1() {
    println!("Running example runtime function");
}

#[hermes_macros::runtime]
async fn example_runtime_f2() -> u8 {
    let x = 4;
    x
}

#[hermes_macros::runtime]
async fn example_runtime_f3() {
    ()
}

#[hermes_macros::runtime]
async fn example_runtime_f4() {
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
}

#[hermes_macros::runtime]
async fn example_runtime_f5() -> Result<u8, Error> {
    let x = 4;
    Ok(x)
}

#[cfg(test)]
mod tests {
    use trybuild::TestCases;

    use super::*;

    #[hermes_macros::test]
    async fn example_test_function() {
        // Example test code to run within the runtime
        println!("Running example test function");
    }

    #[test]
    fn test_compile_failures() {
        let t = TestCases::new();
        t.compile_fail("tests/compile-fail/incorrect_runtime.rs");
    }

    #[test]
    fn test_runtime_macro() {
        assert_eq!(example_runtime_f1(), ());
        assert_eq!(example_runtime_f2(), 4);
        assert_eq!(example_runtime_f3(), ());
        assert_eq!(example_runtime_f4(), ());
        assert_eq!(example_runtime_f5().unwrap(), 4);
    }

    #[test]
    fn test_test_macro() {
        assert_eq!(example_test_function(), ());
    }
}
