use std::io::stdout;
use unicode_width::UnicodeWidthStr; // 添加这个库来处理Unicode字符宽度
use crossterm::{
    execute,
    terminal::{Clear, ClearType},
    event::{KeyCode, KeyModifiers},
    style::Print,
};

use crate::error::FormError;
use crate::input::FormItem;

pub struct TextInput {
    pub prompt: String,
    pub required: bool,
    pub validation: Option<Box<dyn Fn(&str) -> Result<(), String>>>,
    pub value: String,
    pub cursor_pos: usize,
}

impl TextInput {
    pub fn new(
        prompt: &str,
        required: bool,
        validation: Option<Box<dyn Fn(&str) -> Result<(), String>>>,
    ) -> Self {
        TextInput {
            prompt: prompt.to_string(),
            required,
            validation,
            value: String::new(),
            cursor_pos: 0,
        }
    }
    
    fn validate(&self) -> Result<(), FormError> {
        // 只有当用户尝试完成编辑时才验证必填项
        if self.required && self.value.trim().is_empty() {
            return Err(FormError::InvalidInput("此项为必填项，请输入内容!".to_string()));
        }
        
        // 其他验证逻辑保持不变
        if let Some(validator) = &self.validation {
            if let Err(msg) = validator(&self.value) {
                return Err(FormError::InvalidInput(msg));
            }
        }
        
        Ok(())
    }

    // 计算光标在考虑中文宽度的情况下的真实位置
    fn get_display_width(&self, text: &str) -> usize {
        // 使用unicode_width库来计算准确的显示宽度
        UnicodeWidthStr::width(text)
    }
    
    fn get_visual_cursor_pos(&self) -> usize {
        // 修复：确保我们使用字符迭代器而不是字节索引
        let prefix_text: String = self.value.chars().take(self.cursor_pos).collect();
        self.get_display_width(&prefix_text)
    }

    #[allow(dead_code)]
    fn get_cursor_x(&self) -> u16 {
        // 计算表单项前缀宽度 "[_] " 或 "[✓] " = 4 个字符
        let prefix_width = 4;
        
        // 当前编辑状态前缀 "> " = 2 个字符
        let edit_prefix_width = 2;
        
        // 提示文本宽度 - 使用专门的函数考虑中文字符
        let prompt_width = self.get_display_width(&self.prompt);
        
        // 可选标记宽度
        let optional_mark_width = if self.required { 0 } else { 6 }; // " (可选)"
        
        // 冒号和空格分隔符 ": " = 2 个字符
        let separator_width = 2;
        
        // 光标在文本中的可视位置 (考虑中文字符)
        let cursor_pos = self.get_visual_cursor_pos();
        
        // 总光标位置 = 编辑前缀 + 表单状态前缀 + 提示文本 + 可选标记 + 分隔符 + 光标位置
        let total_width = edit_prefix_width + prefix_width + prompt_width + 
                           optional_mark_width + separator_width + cursor_pos;
        
        total_width as u16
    }
}

impl FormItem for TextInput {
    type Output = String;

    fn display(&self) -> Result<(), FormError> {
        execute!(
            stdout(),
            Clear(ClearType::CurrentLine),
            Print(format!(
                "{}{}: {}", 
                self.prompt, 
                if self.required { "" } else { " (可选)" },
                self.value
            ))
        )?;
        
        Ok(())
    }
    
    fn process(&mut self) -> Result<Self::Output, FormError> {
        // 在process中不做任何验证，总是返回当前值
        Ok(self.value.clone())
    }
    
    fn prompt(&self) -> &str {
        &self.prompt
    }
    
    fn is_required(&self) -> bool {
        self.required
    }
    
    fn display_value(&self) -> String {
        self.value.clone()
    }
    
    fn get_type(&self) -> &'static str {
        "text"
    }
    
    fn get_current_value(&self) -> String {
        self.value.clone()
    }
    
    fn handle_key_press(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Result<(), FormError> {
        match key {
            KeyCode::Char(c) if modifiers.is_empty() || modifiers == KeyModifiers::SHIFT => {
                // 支持所有Unicode字符输入，包括中文
                // 使用字符迭代和收集以确保UTF-8安全
                let mut chars: Vec<char> = self.value.chars().collect();
                chars.insert(self.cursor_pos, c);
                self.value = chars.into_iter().collect();
                self.cursor_pos += 1;
            },
            KeyCode::Backspace if self.cursor_pos > 0 => {
                // 删除光标前的字符 - UTF-8安全
                let mut chars: Vec<char> = self.value.chars().collect();
                let new_pos = self.cursor_pos - 1;
                chars.remove(new_pos);
                self.value = chars.into_iter().collect();
                self.cursor_pos = new_pos;
            },
            KeyCode::Delete if self.cursor_pos < self.value.chars().count() => {
                // 删除光标处的字符 - UTF-8安全
                let mut chars: Vec<char> = self.value.chars().collect();
                chars.remove(self.cursor_pos);
                self.value = chars.into_iter().collect();
            },
            KeyCode::Left if self.cursor_pos > 0 => {
                self.cursor_pos -= 1;
            },
            KeyCode::Right if self.cursor_pos < self.value.chars().count() => {
                self.cursor_pos += 1;
            },
            KeyCode::Home => {
                self.cursor_pos = 0;
            },
            KeyCode::End => {
                self.cursor_pos = self.value.chars().count();
            },
            _ => {}
        }
        
        Ok(())
    }
    
    fn move_cursor_left(&mut self) -> Result<(), FormError> {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }
        Ok(())
    }
    
    fn move_cursor_right(&mut self) -> Result<(), FormError> {
        if self.cursor_pos < self.value.chars().count() {
            self.cursor_pos += 1;
        }
        Ok(())
    }
    
    fn complete_editing(&mut self) -> Result<Box<dyn std::any::Any>, FormError> {
        // 只有在这里才执行验证
        self.validate()?;
        Ok(Box::new(self.value.clone()))
    }
    
    fn get_cursor_x(&self) -> u16 {
        self.get_cursor_x()  // 调用自己的方法
    }
}
