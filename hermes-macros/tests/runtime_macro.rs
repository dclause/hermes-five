extern crate hermes_macros;

#[allow(unused_imports)]
use hermes_five::utils::tokio;

#[hermes_macros::runtime]
async fn example_runtime_function() {
    // Example code to run within the runtime
    println!("Running example runtime function");
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
        t.compile_fail("tests/compile_fail/incorrect_runtime.rs");
    }

    #[test]
    fn test_runtime_macro() {
        assert_eq!(example_runtime_function(), ());
    }

    #[test]
    fn test_test_macro() {
        assert_eq!(example_test_function(), ());
    }
}
