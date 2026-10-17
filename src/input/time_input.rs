use std::io::stdout;
use unicode_width::UnicodeWidthStr; // 添加这个库来处理Unicode字符宽度
use crossterm::{
    execute,
    terminal::{Clear, ClearType},
    cursor::MoveTo,  // 保留需要的导入
    style::Print,  // 保留需要的导入
};

use crate::error::FormError;
use crate::input::FormItem;
use crate::models::{Time, TimeRange};

pub struct TimeInput {
    pub prompt: String,
    pub required: bool,
    pub value: Option<TimeRange>,
    pub cursor_pos: usize, // 0=小时开始, 1=分钟开始, 2=小时结束, 3=分钟结束
    pub current_values: [u8; 4], // [小时开始, 分钟开始, 小时结束, 分钟结束]
    pub editing_pos: usize, // 当前正在编辑的数字位置 (0-1)
}

impl TimeInput {
    pub fn new(prompt: &str, required: bool) -> Self {
        TimeInput {
            prompt: prompt.to_string(),
            required,
            value: None,
            cursor_pos: 0,
            current_values: [0, 0, 0, 0],
            editing_pos: 0,
        }
    }
    
    fn get_time_range(&self) -> Result<TimeRange, FormError> {
        let start = Time::new(self.current_values[0], self.current_values[1])?;
        let end = Time::new(self.current_values[2], self.current_values[3])?;
        TimeRange::new(start, end)
    }
    
    fn format_time_value(&self) -> String {
        format!(
            "{:02}:{:02}-{:02}:{:02}",
            self.current_values[0],
            self.current_values[1],
            self.current_values[2],
            self.current_values[3]
        )
    }
    
    fn increase_current_value(&mut self) {
        match self.cursor_pos {
            0 => { // 开始小时
                if self.current_values[0] < 23 {
                    self.current_values[0] += 1;
                } else {
                    self.current_values[0] = 0;
                }
            },
            1 => { // 开始分钟
                if self.current_values[1] < 59 {
                    self.current_values[1] += 1;
                } else {
                    self.current_values[1] = 0;
                }
            },
            2 => { // 结束小时
                if self.current_values[2] < 23 {
                    self.current_values[2] += 1;
                } else {
                    self.current_values[2] = 0;
                }
            },
            3 => { // 结束分钟
                if self.current_values[3] < 59 {
                    self.current_values[3] += 1;
                } else {
                    self.current_values[3] = 0;
                }
            },
            _ => {}
        }
    }
    
    fn decrease_current_value(&mut self) {
        match self.cursor_pos {
            0 => { // 开始小时
                if self.current_values[0] > 0 {
                    self.current_values[0] -= 1;
                } else {
                    self.current_values[0] = 23;
                }
            },
            1 => { // 开始分钟
                if self.current_values[1] > 0 {
                    self.current_values[1] -= 1;
                } else {
                    self.current_values[1] = 59;
                }
            },
            2 => { // 结束小时
                if self.current_values[2] > 0 {
                    self.current_values[2] -= 1;
                } else {
                    self.current_values[2] = 23;
                }
            },
            3 => { // 结束分钟
                if self.current_values[3] > 0 {
                    self.current_values[3] -= 1;
                } else {
                    self.current_values[3] = 59;
                }
            },
            _ => {}
        }
    }
    
    fn set_digit(&mut self, digit: u8) {
        let current_field = &mut self.current_values[self.cursor_pos];
        
        // 处理两位数字的输入
        if self.editing_pos == 0 {
            // 第一位数字
            let max_first_digit = match self.cursor_pos {
                0 | 2 => 2, // 小时首位最大为2
                1 | 3 => 5, // 分钟首位最大为5
                _ => 9,
            };
            
            if digit <= max_first_digit {
                *current_field = digit * 10 + (*current_field % 10);
                self.editing_pos = 1; // 移动到第二位
            }
        } else {
            // 第二位数字
            let max_value = match self.cursor_pos {
                0 | 2 => if *current_field >= 20 { 3 } else { 9 }, // 小时第二位，如果首位是2，则最大为3
                _ => 9,
            };
            
            if digit <= max_value {
                *current_field = (*current_field / 10) * 10 + digit;
                
                // 自动移动到下一个字段 - 修复自动跳过逻辑
                if self.cursor_pos < 3 {
                    self.cursor_pos += 1;
                    self.editing_pos = 0;
                } else {
                    // 已经是最后一个字段，不改变光标位置
                }
            }
        }
    }
    
    fn get_cursor_display_position(&self) -> usize {
        // 获取表单项前缀宽度 "[_] " 或 "[✓] "
        let prefix_width = 4;
        
        // 当前编辑状态前缀 "> " = 2 个字符
        let edit_prefix_width = 2;
        
        // 计算提示文本的宽度，使用准确的Unicode宽度计算
        let prompt_width = UnicodeWidthStr::width(self.prompt.as_str());
        
        // 可选标记的宽度
        let optional_mark_width = if self.required { 0 } else { 6 }; // " (可选)"
        
        // 冒号和空格的宽度 - 注意冒号和空格共占2个字符宽度
        let separator_width = 2; // ": "
        
        // 计算光标在时间字符串中的位置
        // 格式: "HH:MM-HH:MM" (11个字符)
        // 索引:   01234567890
        let time_cursor_pos = match (self.cursor_pos, self.editing_pos) {
            (0, 0) => 0, // 第一组小时十位
            (0, 1) => 1, // 第一组小时个位
            (1, 0) => 3, // 第一组分钟十位
            (1, 1) => 4, // 第一组分钟个位
            (2, 0) => 6, // 第二组小时十位
            (2, 1) => 7, // 第二组小时个位
            (3, 0) => 9, // 第二组分钟十位
            (3, 1) => 10, // 第二组分钟个位
            _ => 0
        };
        
        // 总光标位置 = 前缀 + 编辑状态前缀 + 提示文本 + 可选标记 + 分隔符 + 时间光标位置
        prefix_width + edit_prefix_width + prompt_width + optional_mark_width + separator_width + time_cursor_pos
    }
}

impl FormItem for TimeInput {
    type Output = TimeRange;
    
    fn display(&self) -> Result<(), FormError> {
        let time_str = self.format_time_value();
        
        execute!(
            stdout(),
            Clear(ClearType::CurrentLine),
            Print(format!(
                "{}{}: {}",
                self.prompt,
                if self.required { "" } else { " (可选)" },
                time_str
            ))
        )?;
        
        // 显示提示信息
        let cursor_pos = crossterm::cursor::position()?;
        execute!(
            stdout(), 
            MoveTo(0, cursor_pos.1 + 1),
            Print("使用 ←→ 键移动光标，↑↓ 键更改值，或直接输入数字")
        )?;
        
        Ok(())
    }
    
    fn process(&mut self) -> Result<Self::Output, FormError> {
        match self.get_time_range() {
            Ok(time_range) => {
                self.value = Some(time_range.clone());
                Ok(time_range)
            },
            Err(e) => Err(e)
        }
    }
    
    fn prompt(&self) -> &str {
        &self.prompt
    }
    
    fn is_required(&self) -> bool {
        self.required
    }
    
    fn display_value(&self) -> String {
        self.format_time_value()
    }
    
    // 支持内联编辑的方法
    fn get_type(&self) -> &'static str {
        "time"
    }
    
    fn get_cursor_x(&self) -> u16 {
        // 返回准确的光标位置
        self.get_cursor_display_position() as u16
    }
    
    fn get_current_value(&self) -> String {
        self.format_time_value()
    }
    
    fn move_cursor_left(&mut self) -> Result<(), FormError> {
        // 改进导航逻辑，在各个字段间平滑移动
        if self.editing_pos > 0 {
            self.editing_pos -= 1;
        } else if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            self.editing_pos = 1; // 移到上一组的第二位
        }
        Ok(())
    }
    
    fn move_cursor_right(&mut self) -> Result<(), FormError> {
        // 改进导航逻辑，在各个字段间平滑移动
        if self.editing_pos < 1 {
            self.editing_pos += 1;
        } else if self.cursor_pos < 3 {
            self.cursor_pos += 1;
            self.editing_pos = 0; // 移到下一组的第一位
        }
        Ok(())
    }
    
    fn increase_value(&mut self) -> Result<(), FormError> {
        self.increase_current_value();
        Ok(())
    }
    
    fn decrease_value(&mut self) -> Result<(), FormError> {
        self.decrease_current_value();
        Ok(())
    }
    
    fn input_digit(&mut self, c: char) -> Result<(), FormError> {
        if let Some(digit) = c.to_digit(10) {
            self.set_digit(digit as u8);
        }
        Ok(())
    }
    
    fn complete_editing(&mut self) -> Result<Box<dyn std::any::Any>, FormError> {
        match self.get_time_range() {
            Ok(time_range) => {
                self.value = Some(time_range.clone());
                Ok(Box::new(time_range))
            },
            Err(e) => Err(e)
        }
    }
}
