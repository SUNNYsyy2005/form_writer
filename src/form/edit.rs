use std::io::stdout;  // 删除 Write
use std::thread;
use std::time::Duration;
use crossterm::{
    execute,
    terminal::{Clear, ClearType, size, disable_raw_mode},  // 删除 enable_raw_mode
    cursor::{MoveTo, Show},
    event::{read, Event, KeyCode},  // 删除 KeyModifiers
    style::Print,  // 删除 Attribute, SetAttribute
};
use crate::error::FormError;
use crate::form::types::{Form, ItemState};

impl Form {
    // 主交互循环
    pub fn run_form_loop(&mut self) -> Result<(), FormError> {
        let mut running = true;
        let mut user_canceled = false;
        
        // 计算初始可见项
        let (_, term_height) = size()?;
        self.visible_items = (term_height as usize).saturating_sub(7);
        
        while running {
            self.display()?;
            
            // 处理用户输入
            if let Event::Key(key) = read()? {
                match key.code {
                    // 非编辑状态下的按键处理
                    KeyCode::Up if self.current_index > 0 => {
                        // 只修改当前项的状态，如果它不是已完成状态
                        if self.items[self.current_index].state != ItemState::Completed {
                            self.items[self.current_index].state = ItemState::Normal;
                        }
                        
                        // 移动到上一项
                        self.current_index -= 1;
                        
                        // 设置新当前项为选中状态，除非它已经是完成状态
                        if self.items[self.current_index].state != ItemState::Completed {
                            self.items[self.current_index].state = ItemState::Selected;
                        }
                    },
                    KeyCode::Down if self.current_index < self.items.len() - 1 => {
                        // 只修改当前项的状态，如果它不是已完成状态
                        if self.items[self.current_index].state != ItemState::Completed {
                            self.items[self.current_index].state = ItemState::Normal;
                        }
                        
                        // 移动到下一项
                        self.current_index += 1;
                        
                        // 设置新当前项为选中状态，除非它已经是完成状态
                        if self.items[self.current_index].state != ItemState::Completed {
                            self.items[self.current_index].state = ItemState::Selected;
                        }
                    },
                    KeyCode::Enter => {
                        // 进入编辑模式
                        // 修改：对于已完成的项目，按Enter键可以重新编辑
                        if self.items[self.current_index].state == ItemState::Completed {
                            // 已完成状态变为编辑状态，可以重新编辑
                            self.items[self.current_index].state = ItemState::Editing;
                        } else {
                            self.items[self.current_index].state = ItemState::Editing;
                        }
                        
                        // 处理编辑状态
                        self.handle_editing_mode()?;
                    },
                    KeyCode::Tab => {
                        // 跳到下一个未完成项
                        self.goto_next_incomplete()?;
                    },
                    KeyCode::Esc => {
                        if self.confirm_exit()? {
                            // 保存表单数据到JSON文件 - 修改调用引用到save模块的方法
                            match self.save_form_to_json() {
                                Ok(filename) => {
                                    // 优雅退出：清屏并显示感谢信息及保存路径
                                    execute!(
                                        stdout(), 
                                        Clear(ClearType::All),
                                        MoveTo(0, 0),
                                        Print("感谢使用，表单数据已保存!\n"),
                                        Print(format!("文件保存在: {}\n", filename)),
                                        Print("正在退出...\n")
                                    )?;
                                },
                                Err(e) => {
                                    execute!(
                                        stdout(), 
                                        Clear(ClearType::All),
                                        MoveTo(0, 0),
                                        Print(format!("保存数据失败: {}\n", e)),
                                        Print("正在退出...\n")
                                    )?;
                                }
                            }
                            
                            thread::sleep(Duration::from_millis(1500));
                            running = false;
                            user_canceled = true; // 标记用户取消了表单
                        }
                    },
                    _ => {}
                }
            }
        }
        
        // 恢复终端状态
        disable_raw_mode()?;
        execute!(stdout(), Show, Clear(ClearType::All))?;
        
        // 如果用户取消，返回特殊错误
        if user_canceled {
            return Err(FormError::InvalidInput("用户取消了表单".to_string()));
        }
        
        Ok(())
    }

    // 处理编辑模式
    fn handle_editing_mode(&mut self) -> Result<(), FormError> {
        let mut editing = true;
        
        while editing {
            self.display()?;
            
            if let Event::Key(key) = read()? {
                let current_index = self.current_index;
                let current = &mut self.items[current_index];
                
                match current.item.get_type() {
                    "text" => {
                        if let Ok(completed) = self.handle_text_editing(key.code, key.modifiers) {
                            if completed {
                                editing = false;
                            }
                        }
                    },
                    "single_choice" | "multiple_choice" => {
                        if let Ok(completed) = self.handle_choice_editing(key.code) {
                            if completed {
                                editing = false;
                            }
                        }
                    },
                    "time" => {
                        if let Ok(completed) = self.handle_time_editing(key.code) {
                            if completed {
                                editing = false;
                            }
                        }
                    },
                    _ => {
                        // 未知类型，按ESC退出编辑
                        if key.code == KeyCode::Esc {
                            current.state = ItemState::Selected;
                            editing = false;
                        }
                    }
                }
            }
        }
        
        // 如果当前项目已完成，且不是最后一项，移动到下一项
        if self.items[self.current_index].state == ItemState::Completed && 
           self.current_index < self.items.len() - 1 {
            self.current_index += 1;
            
            // 如果下一项未完成，设为选中状态
            if self.items[self.current_index].state != ItemState::Completed {
                self.items[self.current_index].state = ItemState::Selected;
            }
        }
        
        Ok(())
    }
    
    // 跳转到下一个未完成的表单项
    pub fn goto_next_incomplete(&mut self) -> Result<(), FormError> {
        // 只有当当前项不是已完成状态时才修改其状态
        if self.items[self.current_index].state != ItemState::Completed {
            self.items[self.current_index].state = ItemState::Normal;
        }
        
        // 查找下一个未完成项
        let start_idx = self.current_index;
        let mut idx = (start_idx + 1) % self.items.len();
        
        while idx != start_idx {
            if self.items[idx].state != ItemState::Completed {
                // 更新当前索引
                self.current_index = idx;
                // 设置新的选中状态
                self.items[idx].state = ItemState::Selected;
                return Ok(());
            }
            idx = (idx + 1) % self.items.len();
        }
        
        // 如果所有项都已完成，保持当前选中
        if self.items[self.current_index].state != ItemState::Completed {
            self.items[self.current_index].state = ItemState::Selected;
        }
        Ok(())
    }
    
    // 查找可见项目的位置（新增辅助方法）
    pub(crate) fn find_visible_item_position(&self, idx: usize) -> Result<Option<(usize, u16, u16)>, FormError> {
        // 获取终端尺寸
        let (_, term_height) = size()?;
        let usable_height = term_height.saturating_sub(5);
        
        // 计算实际显示位置
        let mut current_y = self.start_y;
        let mut visible_items = Vec::new();
        
        for i in 0..self.items.len() {
            let item = &self.items[i];
            let mut height = 2; // 默认高度
            
            if item.state == ItemState::Selected || item.state == ItemState::Editing {
                match item.item.get_type() {
                    "single_choice" | "multiple_choice" => {
                        height = item.item.get_options().len() as u16 + 2;
                    },
                    "time" => {
                        height = 3;
                    },
                    _ => {}
                }
            }
            
            if i >= self.scroll_offset && current_y + height <= self.start_y + usable_height {
                visible_items.push((i, current_y, height));
            }
            
            current_y += height + 1;
        }
        
        // 查找指定索引的项
        Ok(visible_items.iter().find(|(i, _, _)| *i == idx).cloned())
    }
}
