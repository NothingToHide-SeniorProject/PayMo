fn main() {
    println!("cargo:rerun-if-changed=src/protos/");

    let proto_files = ["src/protos/example.proto"];

    let mut prost_build = prost_build::Config::new();
    prost_build.protoc_arg("--experimental_allow_proto3_optional");
    prost_build
        .compile_protos(&proto_files, &["src/protos/"])
        .unwrap();
}
