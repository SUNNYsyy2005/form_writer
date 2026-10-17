use std::io::stdout;
use crossterm::{
    execute,
    terminal::size,
    cursor::MoveTo,
    style::{Print, Attribute, SetAttribute},
};
use crate::error::FormError;
use crate::form::types::Form;

impl Form {
    // 渲染表单标题
    pub(crate) fn render_title(&self) -> Result<(), FormError> {
        execute!(
            stdout(),
            SetAttribute(Attribute::Bold),
            Print(format!("===== {} =====\n\n", self.title)),
            SetAttribute(Attribute::Reset)
        )?;
        
        Ok(())
    }

    // 渲染滚动指示器
    pub(crate) fn render_scroll_indicators(&self, has_prev: bool, has_next: bool, next_pos: u16) -> Result<(), FormError> {
        // 显示向上滚动指示器
        if has_prev {
            execute!(stdout(), MoveTo(0, self.start_y - 1), Print("↑ 更多项 ↑"))?;
        }
        
        // 显示向下滚动指示器
        if has_next {
            execute!(stdout(), MoveTo(0, next_pos), Print("↓ 更多项 ↓"))?;
        }
        
        Ok(())
    }

    // 渲染底部导航提示
    pub(crate) fn render_navigation_help(&self) -> Result<(), FormError> {
        let (_, term_height) = size()?;
        
        execute!(
            stdout(), 
            MoveTo(0, term_height - 2),
            Print("使用 ↑↓ 键在表单项之间移动，Enter 进入编辑模式，Tab 跳至下一未完成项，Esc 退出")
        )?;
        
        Ok(())
    }
    
    // 计算字符串显示宽度（考虑中文）
    #[allow(dead_code)]
    pub(crate) fn get_string_width(s: &str) -> usize {
        s.chars().map(|c| if c.is_ascii() { 1 } else { 2 }).sum()
    }
}
