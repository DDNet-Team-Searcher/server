fn main() {
    protobuf_codegen::Codegen::new()
        .include("src/protos")
        .inputs(&["src/protos/request.proto", "src/protos/response.proto"])
        .out_dir("src/protos")
        .run()
        .expect("Yikes");
}
