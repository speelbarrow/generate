let arch = switch variable::get("architecture") {
    "RISC-V" => "risc",
    "Both" => "both",
    _ => "arm",
};
variable::set("arch", arch);

let lib = switch variable::get("library") {
    "embassy (async)" => "async",
    "rp-hal (sync)" => "sync",
    "Both" => "both",
};
variable::set("lib", lib);

if lib != "sync" && arch != "arm" {
    abort("RISC-V architecture is not supported by embassy.");
}

if lib != "both" {
    file::rename(`src/bin/${lib}.rs`, "src/main.rs");
    file::delete("src/bin");
    file::delete("src/lib.rs");
}

if arch == "arm" {
    file::delete("riscv.x");
} else if arch == "risc" {
    file::delete("memory.x");
}
