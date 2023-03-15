fn main() {
    let proto_path = "./proto/";
    let proto_files = ["./proto/example.proto"];

    let mut prost_build = prost_build::Config::new();
    prost_build.protoc_arg("--experimental_allow_proto3_optional");
    prost_build
        .compile_protos(&proto_files, &[proto_path])
        .unwrap();

    println!("cargo:rerun-if-changed={proto_path}");
}
