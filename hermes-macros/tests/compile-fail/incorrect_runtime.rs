extern crate hermes_macros;

#[hermes_macros::runtime]
fn non_async_function() {
    // This should fail because the function is not async
}
