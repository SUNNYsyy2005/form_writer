use std::fmt;
use std::str::FromStr;
use crate::error::FormError;

// 院系枚举 - 展示枚举类型和实现
#[derive(Debug, Clone, PartialEq)]
pub enum Department {
    TeLi,    // 特立书院
    RuiXin,  // 睿信书院
    JingGong // 精工书院
}

impl fmt::Display for Department {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Department::TeLi => write!(f, "特立书院"),
            Department::RuiXin => write!(f, "睿信书院"),
            Department::JingGong => write!(f, "精工书院"),
        }
    }
}

impl FromStr for Department {
    type Err = FormError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1" => Ok(Department::TeLi),
            "2" => Ok(Department::RuiXin),
            "3" => Ok(Department::JingGong),
            _ => Err(FormError::InvalidInput("请选择有效的院系选项".to_string())),
        }
    }
}

// 时间结构体 - 展示结构体和自定义解析逻辑
#[derive(Debug, Clone)]
pub struct Time {
    pub hours: u8,
    pub minutes: u8,
}

impl Time {
    pub fn new(hours: u8, minutes: u8) -> Result<Self, FormError> {
        if hours >= 24 {
            return Err(FormError::RangeError("小时必须在0-23之间".to_string()));
        }
        if minutes >= 60 {
            return Err(FormError::RangeError("分钟必须在0-59之间".to_string()));
        }
        
        Ok(Time { hours, minutes })
    }
    
    // 计算两个时间点之间的分钟差
    pub fn diff_minutes(&self, other: &Time) -> i32 {
        let self_mins = self.hours as i32 * 60 + self.minutes as i32;
        let other_mins = other.hours as i32 * 60 + other.minutes as i32;
        other_mins - self_mins
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:02}:{:02}", self.hours, self.minutes)
    }
}

impl FromStr for Time {
    type Err = FormError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(FormError::FormatError("时间格式应为 HH:MM".to_string()));
        }
        
        let hours = parts[0].parse::<u8>()
            .map_err(|_| FormError::ParseError("小时必须是数字".to_string()))?;
            
        let minutes = parts[1].parse::<u8>()
            .map_err(|_| FormError::ParseError("分钟必须是数字".to_string()))?;
            
        Time::new(hours, minutes)
    }
}

// 时间范围结构体
#[derive(Debug, Clone)]
pub struct TimeRange {
    pub start: Time,
    pub end: Time,
}

impl TimeRange {
    pub fn new(start: Time, end: Time) -> Result<Self, FormError> {
        let diff = start.diff_minutes(&end);
        if diff <= 0 {
            return Err(FormError::RangeError("结束时间必须晚于开始时间".to_string()));
        }
        
        if diff > 360 {  // 6小时 = 360分钟
            return Err(FormError::RangeError("时间范围不能超过6小时".to_string()));
        }
        
        Ok(TimeRange { start, end })
    }
    
    pub fn duration_hours(&self) -> f32 {
        let minutes = self.start.diff_minutes(&self.end);
        minutes as f32 / 60.0
    }
}

impl fmt::Display for TimeRange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{}", self.start, self.end)
    }
}

impl FromStr for TimeRange {
    type Err = FormError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 2 {
            return Err(FormError::FormatError("时间范围格式应为 HH:MM-HH:MM".to_string()));
        }
        
        let start = parts[0].parse::<Time>()?;
        let end = parts[1].parse::<Time>()?;
        
        TimeRange::new(start, end)
    }
}

// 表单数据结构体 - 主要数据结构
#[derive(Debug)]
pub struct StudentForm {
    pub name: String,
    pub student_id: String,
    pub department: Department,
    pub skills: Vec<String>,
    pub courage: u8,
    pub available_time: TimeRange,
    pub high_intensity: bool,
    pub notes: Option<String>,
}

impl StudentForm {
    pub fn display(&self) {
        println!("\n===== 表单提交成功 =====");
        println!("姓名: {}", self.name);
        println!("学号: {}", self.student_id);
        println!("院系: {}", self.department);
        
        println!("特长技能:");
        for (i, skill) in self.skills.iter().enumerate() {
            println!("  {}. {}", i+1, skill);
        }
        
        println!("勇气值: {}/10", self.courage);
        println!("运动时间: {} (共{:.1}小时)", self.available_time, self.available_time.duration_hours());
        println!("需要高强度训练: {}", if self.high_intensity { "是" } else { "否" });
        
        if let Some(notes) = &self.notes {
            println!("备注: {}", notes);
        } else {
            println!("备注: (无)");
        }
        println!("========================");
    }
}
