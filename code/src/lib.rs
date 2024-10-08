use godot::prelude::*;

mod node;

struct RailSystem;

#[gdextension]
unsafe impl ExtensionLibrary for RailSystem {}
