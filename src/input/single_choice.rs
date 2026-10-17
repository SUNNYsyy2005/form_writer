use std::io::stdout;
use crossterm::{
    execute,
    terminal::{Clear, ClearType},
    style::{Print, Attribute, SetAttribute},
};

use crate::error::FormError;
use crate::input::FormItem;

pub struct SingleChoice<T> {
    pub prompt: String,
    pub options: Vec<(String, T)>,
    pub selected_index: Option<usize>,
    pub cursor_pos: usize,
    pub required: bool,
}

impl<T: Clone> SingleChoice<T> {
    pub fn new(prompt: &str, options: Vec<(String, T)>, required: bool) -> Self {
        SingleChoice {
            prompt: prompt.to_string(),
            options,
            selected_index: None,
            cursor_pos: 0,
            required,
        }
    }
    
    fn validate(&self) -> Result<(), FormError> {
        if self.required && self.selected_index.is_none() {
            return Err(FormError::InvalidInput("请选择一个选项".to_string()));
        }
        Ok(())
    }
}

impl<T: Clone + std::fmt::Display +'static> FormItem for SingleChoice<T> {
    type Output = T;
    
    fn display(&self) -> Result<(), FormError> {
        execute!(stdout(), Clear(ClearType::CurrentLine), Print(format!("{}\n", self.prompt)))?;
        
        for (i, (text, _)) in self.options.iter().enumerate() {
            let selected = Some(i) == self.selected_index;
            if i == self.cursor_pos {
                execute!(
                    stdout(),
                    SetAttribute(Attribute::Bold),
                    Print(format!(" > {} [{}] {}\n", i+1, if selected { "X" } else { " " }, text)),
                    SetAttribute(Attribute::Reset)
                )?;
            } else {
                execute!(stdout(), Print(format!("   {} [{}] {}\n", i+1, if selected { "X" } else { " " }, text)))?;
            }
        }
        
        // 显示控制说明
        execute!(
            stdout(),
            Print("\n使用↑↓键移动，空格选择，回车确认\n")
        )?;
        
        Ok(())
    }
    
    fn process(&mut self) -> Result<Self::Output, FormError> {
        if self.options.is_empty() {
            return Err(FormError::InvalidInput("没有可选选项".to_string()));
        }
        
        self.validate()?;
        
        match self.selected_index {
            Some(index) => Ok(self.options[index].1.clone()),
            None => Err(FormError::InvalidInput("未选择任何选项".to_string())),
        }
    }
    
    fn prompt(&self) -> &str {
        &self.prompt
    }
    
    fn is_required(&self) -> bool {
        self.required
    }
    
    fn display_value(&self) -> String {
        match self.selected_index {
            Some(index) => format!("{}", self.options[index].0),
            None => "未选择".to_string(),
        }
    }
    
    // 支持内联编辑的方法
    fn get_type(&self) -> &'static str {
        "single_choice"
    }
    
    fn get_cursor_pos(&self) -> usize {
        self.cursor_pos
    }
    
    fn get_options(&self) -> Vec<String> {
        self.options.iter().map(|(text, _)| text.clone()).collect()
    }
    
    fn is_option_selected(&self, index: usize) -> bool {
        Some(index) == self.selected_index
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
        if self.selected_index == Some(self.cursor_pos) {
            // 取消选择
            self.selected_index = None;
        } else {
            // 设置新选择
            self.selected_index = Some(self.cursor_pos);
        }
        Ok(())
    }
    
    fn get_current_value(&self) -> String {
        match self.selected_index {
            Some(index) => self.options[index].0.clone(),
            None => "未选择".to_string(),
        }
    }
    
    fn complete_editing(&mut self) -> Result<Box<dyn std::any::Any>, FormError> {
        // 如果必填且没有选择，自动选中当前光标位置的选项
        if self.required && self.selected_index.is_none() {
            // 自动选择当前光标位置的选项
            self.selected_index = Some(self.cursor_pos);
        }
        
        // 验证是否选择了选项
        if self.required && self.selected_index.is_none() {
            return Err(FormError::InvalidInput("请选择一个选项".to_string()));
        }
        
        // 返回选择的值或默认值
        match self.selected_index {
            Some(index) => Ok(Box::new(self.options[index].1.clone())),
            None => {
                if !self.required {
                    // 非必填项，返回一个默认标记
                    Ok(Box::new(String::from("[未选择]")))
                } else {
                    // 这里理论上不会执行到，因为前面已经检查过了
                    Err(FormError::InvalidInput("请选择一个选项".to_string()))
                }
            }
        }
    }
}
