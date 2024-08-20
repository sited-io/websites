use std::io::Result;

fn main() -> Result<()> {
    const PROTOS: &[&str] = &[
        "service-apis/proto/sited_io/websites/v1/website.proto",
        "service-apis/proto/sited_io/websites/v1/static_page.proto",
    ];
    const INCLUDES: &[&str] = &["service-apis/proto"];

    tonic_build::configure()
        .out_dir("src/api")
        .protoc_arg("--experimental_allow_proto3_optional")
        .file_descriptor_set_path("src/api/FILE_DESCRIPTOR_SET")
        .build_client(false)
        .build_server(true)
        .type_attribute(".", "#[derive(serde::Deserialize, serde::Serialize)]")
        .compile(PROTOS, INCLUDES)?;

    Ok(())
}
