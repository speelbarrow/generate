@vertex
fn vs_main() -> @builtin(position) vec4<f32> {
    return vec4(0, 0, 0, 0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4(0, 0, 0, 0);
}
