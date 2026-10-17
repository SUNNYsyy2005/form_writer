use std::io::{Write, stdout};
use std::thread;
use std::time::Duration;
use crossterm::{
    execute,
    terminal::size,
    cursor::{MoveTo, Show},
    style::{Print, SetForegroundColor, ResetColor},  // 添加缺失的导入, SetForegroundColor, ResetColor},
};
use crate::error::FormError;
use crate::form::types::{Form, ItemState};

impl Form {
    // 处理文本输入
    pub(crate) fn handle_text_editing(&mut self, key_code: crossterm::event::KeyCode, key_modifiers: crossterm::event::KeyModifiers) 
        -> Result<bool, FormError> {
        
        let current_index = self.current_index;
        
        match key_code {
            crossterm::event::KeyCode::Enter => {
                // 尝试完成编辑
                let current = &mut self.items[current_index];
                match current.item.complete_editing() {
                    Ok(result) => {
                        current.value = Some(result);
                        current.state = ItemState::Completed;
                        Ok(true) // 编辑完成
                    },
                    Err(e) => {
                        self.show_error_message(&e.to_string())?;
                        Ok(false) // 继续编辑
                    }
                }
            },
            crossterm::event::KeyCode::Esc => {
                // 取消编辑
                self.items[current_index].state = ItemState::Selected;
                Ok(false) // 编辑取消
            },
            _ => {
                // 处理其他按键 - 先处理键盘输入
                {
                    let current = &mut self.items[current_index];
                    let _ = current.item.handle_key_press(key_code, key_modifiers);
                } // 可变借用在这里结束
                
                // 不要再调用完整的display()，只更新当前行和光标位置
                if let Some((_, y_pos, _)) = self.find_visible_item_position(current_index)? {
                    // 清除当前行并重新渲染
                    execute!(
                        stdout(),
                        MoveTo(0, y_pos),
                        crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine)
                    )?;
                    
                    let item = &self.items[current_index];
                    
                    // 显示项目前缀
                    let prefix = "> ";  // 编辑模式前缀
                    let status = if item.item.is_required() { "[_] " } else { "[ ] " };
                    execute!(
                        stdout(),
                        MoveTo(0, y_pos),
                        SetForegroundColor(crossterm::style::Color::Green),
                        Print(prefix),
                        Print(status),
                        ResetColor
                    )?;
                    
                    // 渲染文本输入
                    let prompt = item.item.prompt();
                    let optional = if item.item.is_required() { "" } else { " (可选)" };
                    let value = item.item.get_current_value();
                    
                    execute!(
                        stdout(),
                        Print(format!("{}{}: {}", prompt, optional, value))
                    )?;
                    
                    // 计算并设置光标位置
                    let cursor_x = item.item.get_cursor_x();
                    execute!(stdout(), MoveTo(cursor_x, y_pos), Show)?;
                    stdout().flush()?;
                }
                
                // 强制刷新输出确保光标立即更新
                stdout().flush()?;
                
                Ok(false) // 继续编辑
            }
        }
    }

    // 处理选择组件编辑
    pub(crate) fn handle_choice_editing(&mut self, key_code: crossterm::event::KeyCode)
        -> Result<bool, FormError> {
        
        let current = &mut self.items[self.current_index];
        
        match key_code {
            crossterm::event::KeyCode::Up => {
                current.item.move_cursor_up()?;
                Ok(false)
            },
            crossterm::event::KeyCode::Down => {
                current.item.move_cursor_down()?;
                Ok(false)
            },
            crossterm::event::KeyCode::Char(' ') => {
                current.item.toggle_selection()?;
                Ok(false)
            },
            crossterm::event::KeyCode::Enter => {
                match current.item.complete_editing() {
                    Ok(result) => {
                        current.value = Some(result);
                        current.state = ItemState::Completed;
                        Ok(true) // 编辑完成
                    },
                    Err(e) => {
                        self.show_error_message(&e.to_string())?;
                        Ok(false) // 继续编辑
                    }
                }
            },
            crossterm::event::KeyCode::Esc => {
                current.state = ItemState::Selected;
                Ok(false) // 编辑取消
            },
            _ => Ok(false) // 忽略其他键
        }
    }

    // 处理时间输入编辑
    pub(crate) fn handle_time_editing(&mut self, key_code: crossterm::event::KeyCode)
        -> Result<bool, FormError> {
        
        let current_index = self.current_index;
        let current = &mut self.items[current_index];
        
        match key_code {
            crossterm::event::KeyCode::Left => {
                current.item.move_cursor_left()?;
                stdout().flush()?;
                self.update_time_cursor(current_index)?;
            },
            crossterm::event::KeyCode::Right => {
                current.item.move_cursor_right()?;
                stdout().flush()?;
                self.update_time_cursor(current_index)?;
            },
            crossterm::event::KeyCode::Up => {
                current.item.increase_value()?;
                stdout().flush()?;
                self.update_time_cursor(current_index)?;
            },
            crossterm::event::KeyCode::Down => {
                current.item.decrease_value()?;
                stdout().flush()?;
                self.update_time_cursor(current_index)?;
            },
            crossterm::event::KeyCode::Char(c) if c.is_digit(10) => {
                current.item.input_digit(c)?;
                stdout().flush()?;
                self.update_time_cursor(current_index)?;
            },
            crossterm::event::KeyCode::Enter => {
                match current.item.complete_editing() {
                    Ok(result) => {
                        current.value = Some(result);
                        current.state = ItemState::Completed;
                        return Ok(true); // 编辑完成
                    },
                    Err(e) => {
                        self.show_error_message(&e.to_string())?;
                    }
                }
            },
            crossterm::event::KeyCode::Esc => {
                current.state = ItemState::Selected;
                return Ok(false); // 编辑取消
            },
            _ => {}
        }
        
        Ok(false) // 继续编辑
    }
    
    // 更新时间输入的光标位置
    fn update_time_cursor(&self, idx: usize) -> Result<(), FormError> {
        let item = &self.items[idx];
        
        if item.state == ItemState::Editing {
            if let Some((_, y_pos, _)) = self.find_visible_item_position(idx)? {
                let cursor_x = item.item.get_cursor_x();
                
                // 确保光标位置正确，强制刷新
                execute!(stdout(), MoveTo(cursor_x, y_pos))?;
                stdout().flush()?; // 立即刷新光标位置
            }
        }
        
        Ok(())
    }

    // 显示错误消息
    pub(crate) fn show_error_message(&self, message: &str) -> Result<(), FormError> {
        let (_, height) = size()?;
        
        execute!(stdout(), MoveTo(0, height - 4), Print(format!("错误: {}", message)))?;
        thread::sleep(Duration::from_millis(1500));
        execute!(stdout(), MoveTo(0, height - 4), crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine))?;
        
        Ok(())
    }
}
