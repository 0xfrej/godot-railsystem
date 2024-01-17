use godot::prelude::*;

mod scene;

struct RailSystem;

#[gdextension]
unsafe impl ExtensionLibrary for RailSystem {}
