use std::io::Write;
use std::fs::File;
use std::collections::HashMap;
use chrono::Local;
use crate::error::FormError;
use crate::form::types::{Form, ItemState};

// 重新导出保存相关功能，解决命名冲突
// 添加注解允许"未使用"的函数
#[allow(dead_code)]
pub fn save_form_to_json(form: &Form) -> Result<String, FormError> {
    form.save_form_to_json()
}

impl Form {
    // 保存表单数据到JSON文件
    pub fn save_form_to_json(&self) -> Result<String, FormError> {
        // 创建可序列化的数据结构
        let mut form_data = HashMap::new();
        form_data.insert("title".to_string(), self.title.clone());
        
        // 收集表单项数据
        let items_data = self.collect_form_items_data();
        form_data.insert("items".to_string(), format!("{:?}", items_data));
        
        // 生成文件名
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("form_data_{}.json", timestamp);
        
        // 序列化并保存
        let json_data = serde_json::to_string_pretty(&form_data)
            .map_err(|e| FormError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        
        let mut file = File::create(&filename)
            .map_err(|e| FormError::IoError(e))?;
        
        file.write_all(json_data.as_bytes())
            .map_err(|e| FormError::IoError(e))?;
        
        Ok(filename)
    }
    
    // 收集表单项数据
    fn collect_form_items_data(&self) -> Vec<HashMap<String, String>> {
        let mut items_data = Vec::new();
        
        for item in &self.items {
            let mut item_data = HashMap::new();
            item_data.insert("prompt".to_string(), item.item.prompt().to_string());
            item_data.insert("type".to_string(), item.item.get_type().to_string());
            
            if item.state == ItemState::Completed {
                item_data.insert("value".to_string(), item.item.display_value());
                item_data.insert("completed".to_string(), "true".to_string());
            } else {
                item_data.insert("value".to_string(), String::new());
                item_data.insert("completed".to_string(), "false".to_string());
            }
            
            items_data.push(item_data);
        }
        
        items_data
    }
    
    // 导出表单结果到HashMap（可用于不同格式的导出）
    // 添加注解允许"未使用"的方法
    #[allow(dead_code)]
    pub fn export_results(&self) -> HashMap<String, String> {
        let mut results = HashMap::new();
        results.insert("form_title".to_string(), self.title.clone());
        
        for (idx, item) in self.items.iter().enumerate() {
            let key = format!("Q{}_{}",idx + 1, item.item.prompt());
            let value = if item.state == ItemState::Completed {
                item.item.display_value()
            } else {
                "[未完成]".to_string()
            };
            results.insert(key, value);
        }
        
        // 添加时间戳
        results.insert("timestamp".to_string(), 
            Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
        
        results
    }
}
