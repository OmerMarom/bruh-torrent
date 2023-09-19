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
    let (value, len) = parse_internal(content)?;
   
    return
        if len == content.len() {
            Some(value)
        } else {
            None
        }
}

fn parse_internal(content: &str) -> Option<(Value, usize)> {  
    return match content.chars().next()? {
        'l' => parse_list(content).map(|(ls, l)| (Value::List(ls), l)),
        'd' => parse_dictionary(content).map(|(dc, l)| (Value::Dictionary(dc), l)),
        'i' => parse_interger(content).map(|(i, l)| (Value::Integer(i), l)),
        '0'..='9' => parse_string(content).map(|(st, l)| (Value::String(st), l)),
        _ => None
    };
}

fn parse_list(content: &str) -> Option<(Vec<Value>, usize)> {
    let mut list = Vec::new();
    let mut list_len = 2; // Prefix 'l' + suffix 'e'
    let mut content_left = &content[1..];

    loop {
        match content_left.chars().next()? {
            'e' => {
                break;
            }, 
            _ => {
                let (value, value_len) = parse_internal(content_left)?;    
                content_left = &content_left[value_len..];
                list_len += value_len;

                list.push(value);
            }
        };
    }

    Some((list, list_len))
}

fn parse_dictionary(content: &str) -> Option<(HashMap<String, Value>, usize)> {
    let mut dict: HashMap<String, Value> = HashMap::new();
    let mut dict_len = 2; // Prefix 'd' + suffix 'e'
    let mut content_left = &content[1..];

    loop {
        match content_left.chars().next()? {
            'e' => {
                break;
            },
            _ => {
                let (key, key_len) = parse_string(content_left)?;
                content_left = &content_left[key_len..];

                let (value, value_len) = parse_internal(content_left)?;
                content_left = &content_left[value_len..];

                dict_len += key_len + value_len;
                
                dict.insert(key, value);
            }
        };
    } 

    Some((dict, dict_len))
}

fn parse_string(content: &str) -> Option<(String, usize)> {
    let colon_idx = content.find(':')?;
    let length_str = &content[..colon_idx];
    let length = length_str.parse::<usize>().ok()?;
    
    let value_idx = colon_idx + 1;
    let value_end_idx = value_idx + length;
    if value_end_idx > content.len() {
        return None;
    }
    let value = content[value_idx..value_end_idx].to_string();

    Some((value, value_end_idx))
}

fn parse_interger(content: &str) -> Option<(i64, usize)> {
    let e_idx = content.find('e')?;
    let int_str = &content[1..e_idx];
    let int = int_str.parse::<i64>().ok()?;

    Some((int, e_idx + 1))
}

