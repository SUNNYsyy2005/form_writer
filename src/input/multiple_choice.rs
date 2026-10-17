use std::io::stdout;
use crossterm::{
    execute,
    terminal::{Clear, ClearType},
    style::{Print, Attribute, SetAttribute},
};

use crate::error::FormError;
use crate::input::FormItem;

pub struct MultipleChoice<T> {
    pub prompt: String,
    pub options: Vec<(String, T)>, // (显示文本, 对应值)
    pub selected: Vec<bool>,       // 是否选中
    pub cursor_pos: usize,         // 当前光标位置
    pub required: bool,
    pub min_selections: usize,
}

impl< T: Clone > MultipleChoice<T> {
    pub fn new(prompt: &str, options: Vec<(String, T)>, required: bool, min_selections: usize) -> Self {
        MultipleChoice {
            prompt: prompt.to_string(),
            options: options.clone(),
            selected: vec![false; options.len()],
            cursor_pos: 0,
            required,
            min_selections,
        }
    }
    
    fn get_selections(&self) -> Vec<T> {
        self.options.iter()
            .zip(self.selected.iter())
            .filter_map(|((_, value), &selected)| {
                if selected {
                    Some(value.clone())
                } else {
                    None
                }
            })
            .collect()
    }
    
    // 添加注解允许"未使用"的方法
    #[allow(dead_code)]
    fn validate(&self) -> Result<(), FormError> {
        let selected_count = self.selected.iter().filter(|&&x| x).count();
        if self.required && selected_count < self.min_selections {
            return Err(FormError::InvalidInput(format!("请至少选择{}项!", self.min_selections)));
        }
        Ok(())
    }
}

impl<T: Clone + std::fmt::Display +'static> FormItem for MultipleChoice<T> {
    type Output = Vec<T>;

    fn display(&self) -> Result<(), FormError> {
        execute!(stdout(), Clear(ClearType::CurrentLine), Print(format!("{}\n", self.prompt)))?;
        
        for (i, (text, _)) in self.options.iter().enumerate() {
            if i == self.cursor_pos {
                execute!(
                    stdout(),
                    SetAttribute(Attribute::Bold),
                    Print(format!(" > {} [{}] {}\n", i+1, if self.selected[i] { "X" } else { " " }, text)),
                    SetAttribute(Attribute::Reset)
                )?;
            } else {
                execute!(stdout(), Print(format!("   {} [{}] {}\n", i+1, if self.selected[i] { "X" } else { " " }, text)))?;
            }
        }
        
        // 显示控制说明
        execute!(
            stdout(),
            Print("\n使用↑↓键移动，空格选择，回车确认\n"),
            Print(format!("当前已选择: {}项 ", self.selected.iter().filter(|&&x| x).count()))
        )?;
        
        Ok(())
    }
    
    fn process(&mut self) -> Result<Self::Output, FormError> {
        if self.options.is_empty() {
            return Err(FormError::InvalidInput("没有可选选项".to_string()));
        }
        
        // 不要在这里验证，避免立即检查
        Ok(self.get_selections())
    }
    
    fn prompt(&self) -> &str {
        &self.prompt
    }
    
    fn is_required(&self) -> bool {
        self.required
    }
    
    fn display_value(&self) -> String {
        let selections = self.get_selections();
        if selections.is_empty() {
            return "无".to_string();
        }
        
        selections.iter()
            .map(|item| format!("{}", item))
            .collect::<Vec<_>>()
            .join(", ")
    }
    
    // 新增方法，支持内联编辑
    fn get_type(&self) -> &'static str {
        "multiple_choice"
    }
    
    fn get_cursor_pos(&self) -> usize {
        self.cursor_pos
    }
    
    fn get_options(&self) -> Vec<String> {
        self.options.iter().map(|(text, _)| text.clone()).collect()
    }
    
    fn is_option_selected(&self, index: usize) -> bool {
        if index < self.selected.len() {
            self.selected[index]
        } else {
            false
        }
    }
    
    fn move_cursor_up(&mut self) -> Result<(), FormError> {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }
        Ok(())
    }
    
    fn move_cursor_down(&mut self) -> Result<(), FormError> {
        if self.cursor_pos < self.options.len() - 1 {
            self.cursor_pos += 1;
        }
        Ok(())
    }
    
    fn toggle_selection(&mut self) -> Result<(), FormError> {
        if self.cursor_pos < self.selected.len() {
            self.selected[self.cursor_pos] = !self.selected[self.cursor_pos];
        }
        Ok(())
    }
    
    fn get_current_value(&self) -> String {
        let selected_count = self.selected.iter().filter(|&&x| x).count();
        format!("已选择 {} 项", selected_count)
    }
    
    // 添加缺失的方法
    fn complete_editing(&mut self) -> Result<Box<dyn std::any::Any>, FormError> {
        // 检查是否有选择
        let selected_count = self.selected.iter().filter(|&&x| x).count();
        
        // 如果是必填且没有选择足够数量，自动选择当前光标位置
        if self.required && selected_count < self.min_selections {
            // 自动选择当前光标位置的项目
            if self.cursor_pos < self.selected.len() {
                self.selected[self.cursor_pos] = true;
                
                // 重新计算选择数量
                let selected_count = self.selected.iter().filter(|&&x| x).count();
                if selected_count < self.min_selections {
                    return Err(FormError::InvalidInput(format!("请至少选择{}项!", self.min_selections)));
                }
            }
        }
        
        // 始终返回当前选择，即使为空
        Ok(Box::new(self.get_selections()))
    }
}
