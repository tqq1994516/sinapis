fn main() {
    volo_build::ConfigBuilder::default().plugin(volo_build::plugin::SerdePlugin).write().unwrap();
}
