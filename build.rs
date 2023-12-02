fn main() {
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
