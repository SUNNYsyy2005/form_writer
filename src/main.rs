use std::error::Error;
// 删除未使用的导入
// use std::io::{Write, stdout};
// use std::collections::HashSet;

mod error;
mod models;
mod input;
mod form;

// 删除未使用的导入
// use error::FormError;
use models::{Department, StudentForm, Time, TimeRange};
use input::{AnyFormItem, text_input::TextInput, single_choice::SingleChoice, multiple_choice::MultipleChoice, time_input::TimeInput};
use form::Form;

// 表单处理函数 - 使用模块化组件
fn process_form() -> Result<(), Box<dyn Error>> {
    println!("开始创建表单...");
    
    // 创建表单
    let mut form = Form::new("运动员报名表");
    
    // 1. 姓名
    let name_input = TextInput::new("请输入姓名", true, None);
    form.add_item(Box::new(AnyFormItem { inner: name_input }));
    
    // 2. 学号 (11位数字)
    let student_id_validator = Box::new(|input: &str| -> Result<(), String> {
        if input.len() != 11 || !input.chars().all(|c| c.is_digit(10)) {
            return Err("学号必须是11位数字".to_string());
        }
        Ok(())
    });
    
    let student_id_input = TextInput::new("请输入学号", true, Some(student_id_validator));
    form.add_item(Box::new(AnyFormItem { inner: student_id_input }));
    
    // 3. 院系选择
    let departments = vec![
        ("特立书院".to_string(), Department::TeLi),
        ("睿信书院".to_string(), Department::RuiXin),
        ("精工书院".to_string(), Department::JingGong),
    ];
    
    let department_choice = SingleChoice::new("请选择院系", departments, true);
    form.add_item(Box::new(AnyFormItem { inner: department_choice }));
    
    // 4. 特长技能 - 使用多选组件
    let skills_options = vec![
        ("编程".to_string(), "编程".to_string()),
        ("绘画".to_string(), "绘画".to_string()),
        ("写作".to_string(), "写作".to_string()),
        ("演讲".to_string(), "演讲".to_string()),
        ("唱歌".to_string(), "唱歌".to_string()),
        ("运动".to_string(), "运动".to_string()),
        ("摄影".to_string(), "摄影".to_string()),
        ("设计".to_string(), "设计".to_string()),
    ];
    
    let skills_choice = MultipleChoice::new(
        "请选择您的特长技能 (至少选择1项)", 
        skills_options,
        true,
        1
    );
    form.add_item(Box::new(AnyFormItem { inner: skills_choice }));
    
    // 5. 勇气值 (1-10)
    let courage_options = (1..=10)
        .map(|n| (format!("{}/10", n), n))
        .collect();
    
    let courage_choice = SingleChoice::new("请选择您的勇气值", courage_options, true);
    form.add_item(Box::new(AnyFormItem { inner: courage_choice }));
    
    // 6. 可参与运动时间
    let time_input = TimeInput::new("请输入可参与运动时间 (格式: HH:MM-HH:MM)", true);
    form.add_item(Box::new(AnyFormItem { inner: time_input }));
    
    // 7. 是否需要高强度训练
    let high_intensity_options = vec![
        ("是".to_string(), true),
        ("否".to_string(), false),
    ];
    
    let high_intensity_choice = SingleChoice::new("是否需要高强度训练?", high_intensity_options, true);
    form.add_item(Box::new(AnyFormItem { inner: high_intensity_choice }));
    
    // 8. 备注 (可选)
    let notes_input = TextInput::new("请输入备注", false, None);
    form.add_item(Box::new(AnyFormItem { inner: notes_input }));
    
    // 处理表单
    println!("处理表单...");
    match form.process() {
        Ok(results) => {
            // 从结果中提取数据
            let name = results[0].downcast_ref::<String>().unwrap_or(&"".to_string()).clone();
            let student_id = results[1].downcast_ref::<String>().unwrap_or(&"".to_string()).clone();
            let department = results[2].downcast_ref::<Department>().unwrap_or(&Department::TeLi).clone();
            let skills = results[3].downcast_ref::<Vec<String>>().unwrap_or(&vec![]).clone();
            let courage = *results[4].downcast_ref::<u8>().unwrap_or(&0);
            let available_time = results[5].downcast_ref::<TimeRange>().unwrap_or(
                &TimeRange::new(
                    Time::new(0, 0).unwrap(), 
                    Time::new(0, 30).unwrap()
                ).unwrap()
            ).clone();
            let high_intensity = *results[6].downcast_ref::<bool>().unwrap_or(&false);
            let notes_str = results[7].downcast_ref::<String>().unwrap_or(&"".to_string()).clone();
            let notes = if notes_str.trim().is_empty() { None } else { Some(notes_str) };
            
            // 创建表单对象
            let form = StudentForm {
                name,
                student_id,
                department,
                skills,
                courage,
                available_time,
                high_intensity,
                notes,
            };
            
            // 显示表单
            form.display();
            Ok(())
        },
        Err(e) => {
            // 如果是用户取消，正常退出
            if e.to_string().contains("用户取消") {
                println!("表单已取消，退出程序");
                Ok(())
            } else {
                // 其他错误正常返回
                Err(Box::new(e))
            }
        }
    }
}

fn main() {
    println!("开始表单处理...");
    match process_form() {
        Ok(_) => println!("表单处理成功完成!"),
        Err(e) => eprintln!("错误: {}", e),
    }
}