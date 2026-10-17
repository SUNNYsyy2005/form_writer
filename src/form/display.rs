use std::io::{Write, stdout};
use crossterm::{
    execute,
    terminal::{Clear, ClearType, size},
    cursor::{MoveTo, Show},
    event::KeyCode,
    style::{Print, Attribute, SetAttribute, SetForegroundColor, ResetColor},
};
use crate::error::FormError;
use crate::form::types::{Form, ItemState};

impl Form {
    pub fn display(&self) -> Result<(), FormError> {
        // 清屏并重新计算可见区域
        execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0))?;
        
        // 获取终端高度
        let (_, term_height) = size()?;
        
        // 显示标题
        self.render_title()?;

        // 可用的显示区域高度
        let usable_height = term_height.saturating_sub(5); // 减去标题和底部导航的空间
        
        // 计算每个项目的实际显示高度
        let mut item_positions: Vec<(usize, u16, u16)> = Vec::new(); // (索引, y位置, 高度)
        let mut current_y = self.start_y;
        
        // 首先计算所有项目的高度
        for (idx, item_wrapper) in self.items.iter().enumerate() {
            let mut height = 2; // 默认高度
            
            // 根据项目状态和类型计算高度
            if item_wrapper.state == ItemState::Selected || item_wrapper.state == ItemState::Editing {
                match item_wrapper.item.get_type() {
                    "single_choice" | "multiple_choice" => {
                        let options_count = item_wrapper.item.get_options().len();
                        height = options_count as u16 + 2;
                    },
                    "time" => {
                        height = 3;
                    },
                    _ => {}
                }
            }
            
            item_positions.push((idx, current_y, height));
            current_y += height + 1;
        }
        
        // 优先确保当前选中项完全可见的滚动策略
        let mut scroll_offset = 0;
        if let Some(current_pos) = item_positions.iter().position(|(idx, _, _)| *idx == self.current_index) {
            let (_, _, current_height) = item_positions[current_pos];
            
            // 如果当前选中项高度超过可视区域，必须从当前项开始显示
            if current_height >= usable_height {
                scroll_offset = current_pos;
            } else {
                // 尝试调整滚动位置，使当前项完全可见
                // 从当前项向上找，看能显示多少项
                let mut total_height = current_height;
                let mut test_offset = current_pos;
                
                // 向上尝试添加更多项目直到填满屏幕
                while test_offset > 0 && total_height < usable_height {
                    test_offset -= 1;
                    let (_, _, h) = item_positions[test_offset];
                    if total_height + h + 1 <= usable_height {
                        total_height += h + 1;
                    } else {
                        // 这个项目加不进来，恢复索引
                        test_offset += 1;
                        break;
                    }
                }
                
                scroll_offset = test_offset;
            }
        }
        
        // 显示可见项
        let mut visible_items = Vec::new();
        let mut y_pos = self.start_y;
        let mut total_height = 0;
        
        for i in scroll_offset..item_positions.len() {
            let (idx, _, height) = item_positions[i];
            
            // 检查是否还有空间显示这一项
            if total_height + height <= usable_height {
                visible_items.push((idx, y_pos, height));
                y_pos += height + 1;
                total_height += height + 1;
            } else {
                // 空间不足，但如果是当前项，仍尝试尽可能多地显示
                if idx == self.current_index {
                    visible_items.push((idx, y_pos, height));
                }
                break;
            }
        }
        
        // 显示滚动指示器
        let has_prev = scroll_offset > 0;
        let has_next = item_positions.len() > visible_items.len() + scroll_offset;
        let next_pos = if let Some(&(_, y, h)) = visible_items.last() {
            y + h + 1
        } else {
            self.start_y
        };
        
        self.render_scroll_indicators(has_prev, has_next, next_pos)?;
        
        // 显示表单项
        for &(idx, y_pos, _) in &visible_items {
            let item_wrapper = &self.items[idx];
            
            // 修改显示逻辑，支持"已完成但当前选中"状态
            let (prefix, color) = if item_wrapper.state == ItemState::Completed && self.current_index == idx {
                ("> ", crossterm::style::Color::Green)
            } else {
                match item_wrapper.state {
                    ItemState::Normal => ("  ", crossterm::style::Color::Reset),
                    ItemState::Selected => ("> ", crossterm::style::Color::Yellow),
                    ItemState::Editing => ("> ", crossterm::style::Color::Green),
                    ItemState::Completed => ("  ", crossterm::style::Color::Reset),
                }
            };
            
            let status = match item_wrapper.state {
                ItemState::Completed => "[✓] ",
                _ => if item_wrapper.item.is_required() { "[_] " } else { "[ ] " },
            };
            
            // 显示表单项前缀和状态标记，但不再显示提示文本
            execute!(
                stdout(),
                MoveTo(0, y_pos),
                SetForegroundColor(color),
                Print(prefix),
                Print(status),
                ResetColor
            )?;
            
            // 显示不同类型表单项的值或编辑界面
            match item_wrapper.item.get_type() {
                "text" => self.render_text_input(idx, y_pos)?,
                "single_choice" | "multiple_choice" => self.render_choice(idx, y_pos)?,
                "time" => self.render_time_input(idx, y_pos)?,
                _ => {}
            }
        }
        
        // 显示底部导航提示
        self.render_navigation_help()?;
        
        stdout().flush()?;
        
        // 设置光标
        if let Some(wrapper) = self.items.get(self.current_index) {
            if wrapper.state == ItemState::Editing {
                // 获取并记录正确的光标位置
                let edit_x = wrapper.item.get_cursor_x();
                
                // 寻找当前项在可见项中的实际Y坐标
                let edit_y = if let Some(&(_, y_pos, _)) = visible_items.iter().find(|&&(idx, _, _)| idx == self.current_index) {
                    match wrapper.item.get_type() {
                        "text" => y_pos,
                        "single_choice" | "multiple_choice" => {
                            // 选项在下一行，加上光标位置
                            y_pos + 1 + wrapper.item.get_cursor_pos() as u16
                        },
                        "time" => y_pos,
                        _ => y_pos,
                    }
                } else {
                    // 如果项不可见，使用默认位置
                    wrapper.y_pos
                };
                
                // 强制更新光标位置并立即刷新
                execute!(stdout(), MoveTo(edit_x, edit_y), Show)?;
                stdout().flush()?;
            }
        }
        
        Ok(())
    }
    
    
    // 优化版的display_options方法，直接调用render_options
    #[allow(dead_code)]
    pub fn display_options(&self, idx: usize, y_pos: u16, y_offset: u16) -> Result<(), FormError> {
        // 获取可用显示空间
        let (_, term_height) = size()?;
        let usable_height = term_height.saturating_sub(5);
        let remaining_height = (self.start_y + usable_height).saturating_sub(y_pos + y_offset);
        
        let options = self.items[idx].item.get_options();
        let max_options = std::cmp::min(options.len(), remaining_height as usize);
        
        self.render_options(idx, y_pos, max_options, remaining_height)
    }
    
    // 简化版display_item_with_options，使用组件渲染
    #[allow(dead_code)]
    pub fn display_item_with_options(&self, idx: usize) -> Result<(), FormError> {
        let item = &self.items[idx].item;
        
        execute!(stdout(), Clear(ClearType::All))?;
        
        execute!(
            stdout(),
            MoveTo(0, 0),
            SetAttribute(Attribute::Bold),
            Print(format!("正在编辑: {}\n\n", item.prompt())),
            SetAttribute(Attribute::Reset)
        )?;
        
        let options = item.get_options();
        for (i, option) in options.iter().enumerate() {
            let selected = item.is_option_selected(i);
            let marker = if selected { "[X]" } else { "[ ]" };
            let cursor = if i == item.get_cursor_pos() {
                SetAttribute(Attribute::Bold)
            } else {
                SetAttribute(Attribute::Reset)
            };
            
            execute!(
                stdout(),
                MoveTo(2, 2 + i as u16),
                cursor,
                Print(format!("{} {}", marker, option)),
                SetAttribute(Attribute::Reset)
            )?;
        }
        
        // 显示操作指南
        let (_, height) = size()?;
        execute!(
            stdout(), 
            MoveTo(0, height - 3),
            Print("使用 ↑↓ 移动，空格选择，Enter 确认，Esc 取消")
        )?;
        
        Ok(())
    }
    
    // 确认退出对话框
    pub fn confirm_exit(&self) -> Result<bool, FormError> {
        let (width, height) = size()?;
        let message = "确定要退出吗? 未保存的数据将丆失! (Y/N)";
        
        // 保存当前屏幕内容 - 在底部显示确认信息
        let message_y = height - 4; // 在底部导航提示上方显示
        
        // 清除该行并显示提示
        execute!(
            stdout(),
            MoveTo(0, message_y),
            Clear(ClearType::CurrentLine),
            MoveTo((width - message.len() as u16) / 2, message_y),
            SetAttribute(Attribute::Bold),
            Print(message),
            SetAttribute(Attribute::Reset)
        )?;
        
        // 等待按键
        let result = loop {
            match crossterm::event::read()? {
                crossterm::event::Event::Key(key) => match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => break true,
                    KeyCode::Char('n') | KeyCode::Char('N') => break false,
                    KeyCode::Esc => break false, // ESC 也取消退出
                    _ => {}
                },
                _ => {}
            }
        };
        
        // 清除确认信息
        execute!(
            stdout(),
            MoveTo(0, message_y),
            Clear(ClearType::CurrentLine)
        )?;
        
        Ok(result)
    }
}
