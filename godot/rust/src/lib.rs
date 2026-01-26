use godot::init::{ExtensionLibrary, gdextension};

struct Extension;

#[gdextension]
unsafe impl ExtensionLibrary for Extension {}
