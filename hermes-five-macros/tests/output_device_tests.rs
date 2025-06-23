use hermes_five::devices::OutputDevice;
use hermes_five::errors::Error;
use hermes_five::utils::State;
use hermes_five_macros::output_device;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

#[output_device(f32)]
#[derive(Clone, Debug)]
struct TestStruct {
    foobar: bool,
}

impl TestStruct {
    fn parse_state(&self, state: State) -> Result<f32, Error> {
        Ok(match state {
            State::Null => 0f32,
            State::Boolean(false) => 0f32,
            State::Boolean(true) => f32::MAX,
            State::Integer(i) => i as f32,
            State::Signed(s) => s as f32,
            State::Float(f) => f as f32,
            _ => 0f32,
        })
    }

    fn apply_value(&mut self, _: f32) -> Result<(), Error> {
        Ok(())
    }
}

impl Display for TestStruct {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[test]
fn test_macro_usage() {
    // Ce test vérifie que la macro fonctionne en pratique
    let mut test_struct = TestStruct {
        state: Default::default(),
        foobar: true,
        animation: Arc::new(None),
        default: 0.0,
    };

    test_struct.set_state(State::from(42)).unwrap();
    assert_eq!(test_struct.get_value(), 42f32);
    assert_eq!(test_struct.foobar, true);
}
