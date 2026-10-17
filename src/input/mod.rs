use crossterm::event::{KeyCode, KeyModifiers};
use crate::error::FormError;

pub mod text_input;
pub mod single_choice;
pub mod multiple_choice;
pub mod time_input;

// 定义表单项目特质
pub trait FormItem {
    type Output;
    
    // 显示表单项
    fn display(&self) -> Result<(), FormError>;
    
    // 处理输入，返回最终结果
    fn process(&mut self) -> Result<Self::Output, FormError>;
    
    // 获取提示文本
    fn prompt(&self) -> &str;
    
    // 是否为必填项
    fn is_required(&self) -> bool;
    
    // 用于显示当前值的文本表示
    fn display_value(&self) -> String {
        String::new()
    }
    
    // 获取表单项类型
    fn get_type(&self) -> &'static str {
        "unknown"
    }
    
    // 获取光标X坐标（用于内联编辑）
    fn get_cursor_x(&self) -> u16 {
        0
    }
    
    // 获取光标位置
    fn get_cursor_pos(&self) -> usize {
        0
    }
    
    // 获取选项列表（用于选择类表单项）
    fn get_options(&self) -> Vec<String> {
        Vec::new()
    }
    
    // 检查选项是否被选中
    fn is_option_selected(&self, _index: usize) -> bool {
        false
    }
    
    // 获取当前输入值（用于显示）
    fn get_current_value(&self) -> String {
        String::new()
    }
    
    // 处理按键事件
    fn handle_key_press(&mut self, _key: KeyCode, _modifiers: KeyModifiers) -> Result<(), FormError> {
        Ok(())
    }
    
    // 上移光标
    fn move_cursor_up(&mut self) -> Result<(), FormError> {
        Ok(())
    }
    
    // 下移光标
    fn move_cursor_down(&mut self) -> Result<(), FormError> {
        Ok(())
    }
    
    // 左移光标
    fn move_cursor_left(&mut self) -> Result<(), FormError> {
        Ok(())
    }
    
    // 右移光标
    fn move_cursor_right(&mut self) -> Result<(), FormError> {
        Ok(())
    }
    
    // 切换选择状态
    fn toggle_selection(&mut self) -> Result<(), FormError> {
        Ok(())
    }
    
    // 增加数值
    fn increase_value(&mut self) -> Result<(), FormError> {
        Ok(())
    }
    
    // 减少数值
    fn decrease_value(&mut self) -> Result<(), FormError> {
        Ok(())
    }
    
    // 输入数字
    fn input_digit(&mut self, _digit: char) -> Result<(), FormError> {
        Ok(())
    }
    
    // 完成编辑
    fn complete_editing(&mut self) -> Result<Box<dyn std::any::Any>, FormError> {
        Ok(Box::new(()))
    }
}

// 用于统一表单项类型处理
pub struct AnyFormItem<T: FormItem> {
    pub inner: T,
}

impl<T: FormItem + 'static> FormItem for AnyFormItem<T>
where 
    T::Output: 'static
{
    type Output = Box<dyn std::any::Any>;
    
    fn display(&self) -> Result<(), FormError> {
        self.inner.display()
    }
    
    fn process(&mut self) -> Result<Self::Output, FormError> {
        let result = self.inner.process()?;
        Ok(Box::new(result))
    }
    
    fn prompt(&self) -> &str {
        self.inner.prompt()
    }
    
    fn is_required(&self) -> bool {
        self.inner.is_required()
    }
    
    fn display_value(&self) -> String {
        self.inner.display_value()
    }
    
    fn get_type(&self) -> &'static str {
        self.inner.get_type()
    }
    
    fn get_cursor_x(&self) -> u16 {
        self.inner.get_cursor_x()
    }
    
    fn get_cursor_pos(&self) -> usize {
        self.inner.get_cursor_pos()
    }
    
    fn get_options(&self) -> Vec<String> {
        self.inner.get_options()
    }
    
    fn is_option_selected(&self, index: usize) -> bool {
        self.inner.is_option_selected(index)
    }
    
    fn get_current_value(&self) -> String {
        self.inner.get_current_value()
    }
    
    fn handle_key_press(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Result<(), FormError> {
        self.inner.handle_key_press(key, modifiers)
    }
    
    fn move_cursor_up(&mut self) -> Result<(), FormError> {
        self.inner.move_cursor_up()
    }
    
    fn move_cursor_down(&mut self) -> Result<(), FormError> {
        self.inner.move_cursor_down()
    }
    
    fn move_cursor_left(&mut self) -> Result<(), FormError> {
        self.inner.move_cursor_left()
    }
    
    fn move_cursor_right(&mut self) -> Result<(), FormError> {
        self.inner.move_cursor_right()
    }
    
    fn toggle_selection(&mut self) -> Result<(), FormError> {
        self.inner.toggle_selection()
    }
    
    fn increase_value(&mut self) -> Result<(), FormError> {
        self.inner.increase_value()
    }
    
    fn decrease_value(&mut self) -> Result<(), FormError> {
        self.inner.decrease_value()
    }
    
    fn input_digit(&mut self, digit: char) -> Result<(), FormError> {
        self.inner.input_digit(digit)
    }
    
    fn complete_editing(&mut self) -> Result<Box<dyn std::any::Any>, FormError> {
        // 不再尝试调用process方法，直接返回已完成
        match self.inner.complete_editing() {
            Ok(value) => Ok(value),
            Err(e) => Err(e),
        }
    }
}
