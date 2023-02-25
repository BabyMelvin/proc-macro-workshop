/// Note: 这里需要pub 导入Builder，test模块才能Import成功
pub use derive_builder::Builder;

// `cargo expand` 使用

#[derive(Builder)]
pub struct Command {
    executable: String,
    args: Vec<String>,
    env: Vec<String>,
    current_dir: String,
}