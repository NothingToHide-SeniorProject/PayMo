fn main() {
    println!("cargo:rerun-if-changed=src/protos/");

    let proto_files = ["src/protos/example.proto"];
    prost_build::compile_protos(&proto_files, &["src/protos/"]).unwrap();
}
