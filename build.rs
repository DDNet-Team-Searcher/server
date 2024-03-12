use std::path::Path;

fn main() {
    if !Path::new("src/protos").exists() {
        let _ = std::fs::create_dir("src/protos").expect("Failed to create directory");
    }

    protobuf_codegen::Codegen::new()
        .include("protos")
        .inputs(&[
            "protos/request.proto",
            "protos/response.proto",
            "protos/common.proto",
        ])
        .out_dir("src/protos")
        .run()
        .expect("Yikes");
}
