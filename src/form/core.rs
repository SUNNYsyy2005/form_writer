use std::any::Any;
use std::io::stdout;  // 删除 Write
// 删除未使用的导入
// use std::fs;
// use std::collections::HashMap;
use crossterm::{
    execute,
    terminal::{Clear, ClearType, enable_raw_mode},  // 删除 disable_raw_mode
    // 删除 cursor::Show
};
use crate::error::FormError;
use crate::form::types::{Form, FormItemWrapper, ItemState};
use crate::input::FormItem;
// 删除未使用的导入
// use super::display;
// use super::edit;

impl Form {
    pub fn new(title: &str) -> Self {
        Form {
            items: Vec::new(),
            title: title.to_string(),
            current_index: 0,
            start_y: 2,       // 标题下方两行开始
            scroll_offset: 0, // 初始无滚动
            visible_items: 0, // 初始时计算
        }
    }
    
    pub fn add_item(&mut self, item: Box<dyn FormItem<Output = Box<dyn Any>>>) {
        // 初始状态下的高度，选项类控件使用折叠状态高度
        let base_height = match item.get_type() {
            "text" => 2,
            "single_choice" | "multiple_choice" => 2, // 未选中时只显示一行
            "time" => 3,
            _ => 2,
        };
        
        // 计算展开后的高度，用于内部记录
        let _expanded_height = match item.get_type() {  // 添加下划线前缀
            "single_choice" | "multiple_choice" => {
                // 展开后需要显示所有选项
                let options_count = item.get_options().len() as u16;
                options_count + 3 // 标题行 + 选项 + 提示行
            },
            _ => base_height
        };
        
        let y_pos = if self.items.is_empty() {
            self.start_y
        } else {
            let last_item = self.items.last().unwrap();
            last_item.y_pos + last_item.height + 1
        };
        
        self.items.push(FormItemWrapper {
            item,
            state: ItemState::Normal,
            value: None,
            y_pos,
            height: base_height, // 初始使用基本高度
        });
    }
    
    pub fn process(&mut self) -> Result<Vec<Box<&dyn Any>>, FormError> {
        // 清屏并启用原始模式
        execute!(stdout(), Clear(ClearType::All))?;
        enable_raw_mode()?;
        
        // 设置初始选中项的状态
        if !self.items.is_empty() {
            self.items[0].state = ItemState::Selected;
        }
        
        // 进入交互循环
        match self.run_form_loop() {
            Ok(_) => {
                // 用户完成了表单，收集和返回结果
                let mut results = Vec::new();
                for item_wrapper in &self.items {
                    if let Some(val) = &item_wrapper.value {
                        results.push(Box::new(val.as_ref()));
                    } else {
                        results.push(Box::new(&()));
                    }
                }
                
                Ok(results)
            },
            Err(e) => {
                // 用户取消了表单或发生错误
                Err(e)
            }
        }
    }
    
    // 添加注解允许"未使用"的函数
    #[allow(dead_code)]
    pub fn all_completed(&self) -> bool {
        self.items.iter().all(|item| {
            !item.item.is_required() || item.state == ItemState::Completed
        })
    }
    
    // 删除重复的 export_results 函数，保留在 save.rs 中的版本
}
