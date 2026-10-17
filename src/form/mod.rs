mod types;
mod core;

// 显示相关模块
mod display;
mod render;
mod components;

// 编辑相关模块
mod edit;
mod handlers;
mod save;

// 重新导出核心类型和函数
pub use types::Form;
// 删除未使用的导出
// pub use types::ItemState;
// pub use save::save_form_to_json;
