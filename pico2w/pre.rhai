let arch = switch variable::get("architecture") {
    "RISC-V" => "risc",
    "Both" => "both",
    _ => "arm",
};
variable::set("arch", arch);

let lib = switch variable::get("library") {
    "embassy (async)" => "async",
    "rp-hal (sync)" => "sync",
};
variable::set("lib", lib);

let wifi = if variable::is_set("wifi") {
    variable::get("wifi")
} else {
    let r = false;
    variable::set("wifi", r);
    r
};

if lib != "sync" && arch != "arm" {
    abort("RISC-V architecture is not supported by embassy.");
}

file::rename(`src/main/${lib}.rs`, "src/main.rs");
file::delete("src/main");

if arch == "arm" {
    file::delete("riscv.x");
} else if arch == "risc" {
    file::delete("memory.x");
}

if !wifi {
    file::delete("43439A0.bin");
    file::delete("43439A0_clm.bin");
    file::delete("nvram_rp2040.bin");
    file::delete("src/wifi.rs")
}
