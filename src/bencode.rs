use std::{fmt, str, collections::HashMap};

pub enum Value<'a> {
    Integer(u64),
    ByteString(Vec<u8>),
    List(Vec<Node<'a>>),
    Dictionary(HashMap<String, Node<'a>>),
}

struct Node<'a> {
    pub value: Value<'a>,
    pub unparsed: &'a [u8],
}

impl<'a> Node<'a> {
    pub fn as_integer(&self) -> Option<u64> {
        if let Value::Integer(i) = self.value {
            Some(i) 
        } else {
            None
        }
    }

    pub fn as_byte_string(&self) -> Option<&Vec<u8>> {
        if let Value::ByteString(bs) = self.value {
            Some(&bs) 
        } else {
            None
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        str::from_utf8(self.as_byte_string()?).ok()
    }

    pub fn as_list(&self) -> Option<Vec<Node>> {
        if let Value::List(l) = self.value {
            Some(l) 
        } else {
            None
        }
    }

    pub fn as_dictionary(&self) -> Option<HashMap<String, Node>> {
        if let Value::Dictionary(d) = self.value {
            Some(d) 
        } else {
            None
        }
    }
}

impl<'a> fmt::Display for Node<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            Value::ByteString(s) => write!(
                f,
                "\"{}\"",
                str::from_utf8(&s).unwrap_or("[non-utf8 byte string]")
            ),
            Value::Integer(i) => write!(f, "Integer: {}", i),
            Value::List(l) => write!(
                f,
                "[{}]",
                l
                    .iter()
                    .map(|value| value.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Value::Dictionary(d) => write!(
                f,
                "{{\n{} }}",
                d
                    .iter()
                    .map(|(key, value)| format!("\t{}: {}", key, value))
                    .collect::<Vec<String>>()
                    .join(", \n")
            )
        }
    }
}

pub fn parse(content: &[u8]) -> Option<Node> {
    let (value, parse_len) = parse_value(content)?;
   
    if parse_len == content.len() {
        Some(Node { value, unparsed: content })
    } else {
        None
    }
}

fn parse_value(content: &[u8]) -> Option<(Value, usize)> {  
    match content[0] {
        b'l' => parse_list(content).map(|(l, len)| (Value::List(l), len)),
        b'd' => parse_dictionary(content).map(|(d, len)| (Value::Dictionary(d), len)),
        b'i' => parse_interger(content).map(|(i, len)| (Value::Integer(i), len)),
        b'0'..=b'9' => parse_byte_string(content).map(|(bs, s)| (Value::ByteString(bs), s)),
        _ => None
    }
}

fn parse_list(content: &[u8]) -> Option<(Vec<Node>, usize)> {
    let mut list = Vec::new();
    let mut unparsed = &content[1..];
    let mut parse_len = 1;

    loop {
        match unparsed[0] {
            b'e' => {
                parse_len += 1;
                break;
            }, 
            _ => {
                let (value, value_parse_len) = parse_value(unparsed)?;    
                list.push(Node { value, unparsed: &unparsed[..value_parse_len] });
                unparsed = &unparsed[value_parse_len..];
            }
        };
    }

    Some((list, parse_len))
}

fn parse_dictionary(content: &[u8]) -> Option<(HashMap<String, Node>, usize)> {
    let mut dict = HashMap::new();
    let mut unparsed = &content[1..];    
    let mut parse_len = 1;

    loop {
        match unparsed[0] {
            b'e' => {
                parse_len += 1;
                break;
            },
            _ => {
                let (key_byte_str, key_parse_len) = parse_byte_string(unparsed)?;
                let key = str::from_utf8(&key_byte_str).ok()?.to_string();
                unparsed = &unparsed[key_parse_len..];
                let (value, value_parse_len) = parse_value(unparsed)?;
                dict.insert(key, Node { value, unparsed: &unparsed[..value_parse_len] });
                unparsed = &unparsed[value_parse_len..];
            }
        };
    } 

    Some((dict, parse_len))
}

fn parse_byte_string(content: &[u8]) -> Option<(Vec<u8>, usize)> {
    let colon_idx = content.iter().position(|c| c == &b':')?;
    let length = str::from_utf8(&content[..colon_idx]).ok()?.parse::<usize>().ok()?;
    
    let value_idx = colon_idx + 1;
    let value_end_idx = value_idx + length;
    if value_end_idx > content.len() {
        return None;
    }
    let value = content[value_idx..value_end_idx].to_vec();

    Some((value, value_end_idx))
}

fn parse_interger(content: &[u8]) -> Option<(u64, usize)> {
    let e_idx = content.iter().position(|c| c == &b'e')?;
    let int = str::from_utf8(&content[1..e_idx]).ok()?.parse::<u64>().ok()?;
    
    Some((int, e_idx + 1))
}

