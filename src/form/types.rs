use std::any::Any;
use crate::input::FormItem;

// 表单项的状态
#[derive(PartialEq)]
pub enum ItemState {
    Normal,      // 普通状态
    Selected,    // 选中未编辑
    Editing,     // 正在编辑
    Completed,   // 已完成
}

pub struct FormItemWrapper {
    pub item: Box<dyn FormItem<Output = Box<dyn Any>>>,
    pub state: ItemState,
    pub value: Option<Box<dyn Any>>,
    pub y_pos: u16,   // 在表单中的纵向位置
    pub height: u16,  // 表单项高度
}

pub struct Form {
    pub items: Vec<FormItemWrapper>,
    pub title: String,
    pub current_index: usize,
    pub start_y: u16,         // 表单开始的y坐标
    pub scroll_offset: usize, // 滚动偏移量
    pub visible_items: usize, // 可见项的数量
}
