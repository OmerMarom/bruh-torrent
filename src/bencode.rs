use std::collections::HashMap;
use std::fmt;

pub enum Value {
    Integer(i64),
    String(String),
    List(Vec<Value>),
    Dictionary(HashMap<String, Value>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::String(s) => write!(f, "String: {}", s),
            Value::Integer(i) => write!(f, "Integer: {}", i),
            Value::List(l) => write!(
                f,
                "List: {}",
                l
                    .iter()
                    .map(|value| value.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Value::Dictionary(d) => write!(
                f,
                "Dict: {}",
                d
                    .iter()
                    .map(|(key, value)| format!("{}: {}", key, value))
                    .collect::<Vec<String>>()
                    .join(", ")
            )
        }
    }
}

pub fn parse(content: &str) -> Option<Value> {
    let (value, content_left) = parse_value(content)?;
   
    if content_left.is_empty() {
        Some(value)
    } else {
        None
    }
}

fn parse_value(content: &str) -> Option<(Value, &str)> {  
    match content.chars().next()? {
        'l' => parse_list(content).map(|(l, cl)| (Value::List(l), cl)),
        'd' => parse_dictionary(content).map(|(d, cl)| (Value::Dictionary(d), cl)),
        'i' => parse_interger(content).map(|(i, cl)| (Value::Integer(i), cl)),
        '0'..='9' => parse_string(content).map(|(s, cl)| (Value::String(s), cl)),
        _ => None
    }
}

fn parse_list(content: &str) -> Option<(Vec<Value>, &str)> {
    let mut list = Vec::new();
    let mut content_left = &content[1..];

    loop {
        match content_left.chars().next()? {
            'e' => {
                content_left = &content_left[1..];
                break;
            }, 
            _ => {
                let (value, content_after_value) = parse_value(content_left)?;    
                content_left = content_after_value;
                list.push(value);
            }
        };
    }

    Some((list, content_left))
}

fn parse_dictionary(content: &str) -> Option<(HashMap<String, Value>, &str)> {
    let mut dict: HashMap<String, Value> = HashMap::new();
    let mut content_left = &content[1..];

    loop {
        match content_left.chars().next()? {
            'e' => {
                content_left = &content_left[1..];
                break;
            },
            _ => {
                let (key, content_after_key) = parse_string(content_left)?;
                let (value, content_after_value) = parse_value(content_after_key)?;
                content_left = content_after_value;
                dict.insert(key, value);
            }
        };
    } 

    Some((dict, content_left))
}

fn parse_string(content: &str) -> Option<(String, &str)> {
    let colon_idx = content.find(':')?;
    let length = content[..colon_idx].parse::<usize>().ok()?;
    
    let value_idx = colon_idx + 1;
    let value_end_idx = value_idx + length;
    if value_end_idx > content.len() {
        return None;
    }
    let value = content[value_idx..value_end_idx].to_string();

    Some((value, &content[value_end_idx..]))
}

fn parse_interger(content: &str) -> Option<(i64, &str)> {
    let e_idx = content.find('e')?;
    let int = content[1..e_idx].parse::<i64>().ok()?;

    Some((int, &content[e_idx + 1..]))
}

