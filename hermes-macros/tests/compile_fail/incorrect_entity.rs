#![cfg(feature = "serde")]

extern crate hermes_macros;

use hermes_macros::Entity;

#[Entity]
struct InvalidEntity {
    id: u8, // reserved
    name: String,
}
