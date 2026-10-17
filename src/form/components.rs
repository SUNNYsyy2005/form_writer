use std::io::stdout;
use crossterm::{
    execute,
    terminal::size,
    cursor::MoveTo,
    style::{Print, Attribute, SetAttribute},
};
use crate::error::FormError;
use crate::form::types::{Form, ItemState};

impl Form {
    // 渲染文本输入组件 - 确保与光标计算保持一致
    pub(crate) fn render_text_input(&self, idx: usize, y_pos: u16) -> Result<(), FormError> {
        let item_wrapper = &self.items[idx];
        let prompt = item_wrapper.item.prompt();
        let optional = if item_wrapper.item.is_required() { "" } else { " (可选)" };
        
        // 统一执行渲染文本，确保格式与cursor_x计算一致
        execute!(
            stdout(), 
            Print(format!("{}{}: ", prompt, optional))
        )?;
        
        // 保留值的渲染逻辑，确保一致性
        if item_wrapper.state == ItemState::Editing {
            let value = item_wrapper.item.get_current_value();
            execute!(stdout(), Print(value))?;
        } else if item_wrapper.state == ItemState::Completed {
            let value = item_wrapper.item.display_value();
            execute!(stdout(), Print(value))?;
            if self.current_index == idx {
                execute!(stdout(), MoveTo(4, y_pos + 1), Print("(已完成，按Enter可重新编辑)"))?;
            }
        } else {
            execute!(stdout(), Print("_______________"))?;
        }
        
        Ok(())
    }

    // 渲染选择组件
    pub(crate) fn render_choice(&self, idx: usize, y_pos: u16) -> Result<(), FormError> {
        let item_wrapper = &self.items[idx];
        let prompt = item_wrapper.item.prompt();
        let optional = if item_wrapper.item.is_required() { "" } else { " (可选)" };
        
        // 先渲染提示文本
        execute!(stdout(), Print(format!("{}{}: ", prompt, optional)))?;
        
        if item_wrapper.state == ItemState::Completed {
            let value = item_wrapper.item.display_value();
            execute!(stdout(), Print(value))?;
            
            if self.current_index == idx {
                execute!(stdout(), MoveTo(4, y_pos + 1), Print("(已完成，按Enter可重新编辑)"))?;
            }
        } else if item_wrapper.state == ItemState::Selected || item_wrapper.state == ItemState::Editing {
            // 获取可用显示空间
            let (_, term_height) = size()?;
            let usable_height = term_height.saturating_sub(5);
            
            // 显示选项
            let options = item_wrapper.item.get_options();
            let remaining_height = (self.start_y + usable_height).saturating_sub(y_pos + 1);
            let max_options = std::cmp::min(options.len(), remaining_height as usize);
            
            self.render_options(idx, y_pos, max_options, remaining_height)?;
        } else {
            execute!(stdout(), Print("[选择项]"))?;
        }
        
        Ok(())
    }

    // 渲染时间输入组件
    pub(crate) fn render_time_input(&self, idx: usize, y_pos: u16) -> Result<(), FormError> {
        let item_wrapper = &self.items[idx];
        let prompt = item_wrapper.item.prompt();
        let optional = if item_wrapper.item.is_required() { "" } else { " (可选)" };
        
        // 先渲染提示文本
        execute!(stdout(), Print(format!("{}{}: ", prompt, optional)))?;
        
        // 然后处理不同状态
        if item_wrapper.state == ItemState::Editing {
            let value = item_wrapper.item.get_current_value();
            execute!(stdout(), Print(value))?;
            
            execute!(
                stdout(), 
                MoveTo(4, y_pos + 1),
                Print("使用 ←→ 键移动光标，↑↓ 键修改值，或直接输入数字")
            )?;
        } else if item_wrapper.state == ItemState::Completed {
            let value = item_wrapper.item.display_value();
            execute!(stdout(), Print(value))?;
            
            if self.current_index == idx {
                execute!(stdout(), MoveTo(4, y_pos + 1), Print("(已完成，按Enter可重新编辑)"))?;
            }
        } else {
            execute!(stdout(), Print("__:__ - __:__"))?;
        }
        
        Ok(())
    }

    // 渲染选项列表
    pub(crate) fn render_options(&self, idx: usize, y_pos: u16, max_options: usize, remaining_height: u16) -> Result<(), FormError> {
        let item_wrapper = &self.items[idx];
        let options = item_wrapper.item.get_options();
        
        for (i, option) in options.iter().take(max_options).enumerate() {
            let marker = if item_wrapper.item.is_option_selected(i) { "[X]" } else { "[ ]" };
            let option_style = if item_wrapper.state == ItemState::Editing && item_wrapper.item.get_cursor_pos() == i {
                SetAttribute(Attribute::Bold)
            } else {
                SetAttribute(Attribute::Reset)
            };
            
            execute!(
                stdout(),
                MoveTo(4, y_pos + 1 + i as u16),
                option_style,
                Print(format!("{} {}", marker, option)),
                SetAttribute(Attribute::Reset)
            )?;
        }
        
        // 显示更多项提示
        if options.len() > max_options {
            execute!(
                stdout(),
                MoveTo(4, y_pos + 1 + max_options as u16),
                Print("... 还有更多选项 ...")
            )?;
        }
        
        // 显示操作提示
        if max_options > 0 {
            let hint_y = y_pos.saturating_add(1).saturating_add(
                std::cmp::min(max_options as u16, remaining_height.saturating_sub(1))
            );
            
            if hint_y < self.start_y + (size()?.1).saturating_sub(5) {
                execute!(
                    stdout(),
                    MoveTo(4, hint_y),
                    Print("使用 ↑↓ 移动，空格选择，Enter 确认")
                )?;
            }
        }
        
        Ok(())
    }
}
