use std::{
    env,
    path::{Path, PathBuf},
};

fn main() {
    let proto_path = "./proto/";
    let proto_files = ["./proto/paymo.proto"];

    let mut prost_build = prost_build::Config::new();
    prost_build.protoc_arg("--experimental_allow_proto3_optional");
    prost_build
        .compile_protos(&proto_files, &[proto_path])
        .unwrap();

    println!("cargo:rerun-if-changed={proto_path}");
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rustc-link-lib=gmp");

    let include_dir = Path::new("./liblhtlp/include");

    cc::Build::new()
        .warnings(false)
        .file("./liblhtlp/src/lhp.c")
        .file("./liblhtlp/src/params.c")
        .file("./liblhtlp/src/puzzle.c")
        .file("./liblhtlp/src/util.c")
        .include(include_dir)
        .compile("lhtlp");

    let bindings = bindgen::Builder::default()
        .ctypes_prefix("cty")
        .header("./wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .unwrap();

    let bindings_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(bindings_path.join("bindings.rs"))
        .unwrap();
}
