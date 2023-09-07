fn main() -> Result<(), std::io::Error> {
    tonic_build::configure().build_server(true).compile(
        &[
            "proto/user.proto",
            "proto/report.proto",
        ],
        &["share_resources/proto"],
    )?;
    Ok(())
}