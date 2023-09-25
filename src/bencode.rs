use std::{fmt, str, collections::HashMap};
use thiserror::Error;

pub enum Value<'a> {
    Integer(i64),
    ByteString(Vec<u8>),
    List(Vec<Node<'a>>),
    Dictionary(HashMap<String, Node<'a>>),
}

pub struct Node<'a> {
    pub value: Value<'a>,
    pub unparsed: &'a [u8],
}

impl<'a> Node<'a> {
    pub fn as_integer(&self) -> Option<i64> {
        if let Value::Integer(i) = self.value {
            Some(i) 
        } else {
            None
        }
    }

    pub fn as_byte_string(&self) -> Option<&Vec<u8>> {
        if let Value::ByteString(bs) = &self.value {
            Some(bs) 
        } else {
            None
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        str::from_utf8(self.as_byte_string()?).ok()
    }

    pub fn as_list(&self) -> Option<&Vec<Node>> {
        if let Value::List(l) = &self.value {
            Some(l) 
        } else {
            None
        }
    }

    pub fn as_dictionary(&self) -> Option<&HashMap<String, Node>> {
        if let Value::Dictionary(d) = &self.value {
            Some(d) 
        } else {
            None
        }
    }
}

impl<'a> fmt::Display for Node<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.value {
            Value::Integer(i) => write!(f, "Integer: {}", i),
            Value::ByteString(s) => write!(
                f,
                "\"{}\"",
                str::from_utf8(&s).unwrap_or("[non-utf8 byte string]")
            ),
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
                "{{\n{} \n}}",
                d
                    .iter()
                    .map(|(key, value)| format!("\t{}: {}", key, value))
                    .collect::<Vec<String>>()
                    .join(", \n")
            )
        }
    }
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Unexpected data after Bencode")]
    DataAfterBencode,
    #[error("Unexpected end of data")]
    UnexpectedEndOfData,
    #[error("Invalid prefix")]
    InvalidPrefix,
    #[error("Unexpected non-utf8 integer")]
    NonUtf8Integer,
    #[error("Unexpected non-utf8 dictionary key")]
    NonUtf8DictKey,
    #[error("Invalid integer")]
    InvalidInteger,
    #[error("Non byte string dictionary key")]
    NonByteStringDictKey
}

// TODO This parser doesn't handle whitespaces.
// fn trim_start(content: &[u8]) -> (&[u8], usize) {
//    let whitespaces = content.iter().position(|&c| c != b' ').unwrap_or(content.len());
//
//    (&content[whitespaces..], whitespaces)
// }

pub fn parse(content: &[u8]) -> Result<Node, ParseError> {
    let (value, parse_len) = parse_value(content)?;
   
    if parse_len == content.len() {
        Ok(Node { value, unparsed: content })
    } else {
        Err(ParseError::DataAfterBencode)
    }
}

fn parse_value(content: &[u8]) -> Result<(Value, usize), ParseError> {
    if content.is_empty() {
        Err(ParseError::UnexpectedEndOfData)       
    } else {
        parse_interger(content).map(|ri| ri.map(|(i, l)| (Value::Integer(i), l)))
            .or(parse_byte_string(content).map(|rbs| rbs.map(|(bs, l)| (Value::ByteString(bs), l))))
            .or(parse_list(content).map(|rl| rl.map(|(ls, l)| (Value::List(ls), l))))
            .or(parse_dictionary(content).map(|rd| rd.map(|(d, l)| (Value::Dictionary(d), l))))
            .unwrap_or(Err(ParseError::InvalidPrefix))
    }
}

// Note about all parse_<type> functions:
// - They all expect non-empty inputs.
// - They all return Option<Result>:
//   If None is returned, that means the content's prefix doesn't match the type's, 
//   meaning the content does not represent a value of that type.
//   If Some is returned, if the result is an error, that means that the content does 
//   represent this type, but it contains invalid bencode syntax.
//   Otherwise, the parsed value is returned in the Ok branch of the Result.

fn parse_interger(content: &[u8]) -> Option<Result<(i64, usize), ParseError>> {
    if content[0] != b'i' {
        return None;
    };

    // TODO A better approach may be to iterate over the string until a non-number 
    //      or leading dash is encountered: If the character is not an 'e',
    //      the integer is invalid and we can return an error.

    let e_idx = match content.iter().position(|c| c == &b'e') {
        Some(e_idx) => e_idx,
        None => return Some(Err(ParseError::UnexpectedEndOfData))
    };
    let int_slice = &content[1..e_idx];
    
    let int_str = match str::from_utf8(int_slice) {
        Err(_err) => return Some(Err(ParseError::NonUtf8Integer)),
        Ok(int_str) => int_str,
    };
    
    let int = match int_str.parse::<i64>() {
        Err(_err) => return Some(Err(ParseError::InvalidInteger)),
        Ok(int) => int,
    };

    Some(Ok((int, e_idx + 1)))
}

fn parse_byte_string(content: &[u8]) -> Option<Result<(Vec<u8>, usize), ParseError>> {
    // TODO A better approach may be to iterate over the string until a non-number is 
    //      encountered: If the character is not a colon, the prefix is invalid and we 
    //      can return None.

    // Quick way to check if this is probably a string:
    let first_char = content[0];
    if first_char <= b'0' || first_char >= b'9' {
        return None;
    };

    let colon_idx = content.iter().position(|c| c == &b':')?;
    let length_slice = &content[..colon_idx];
    let length = str::from_utf8(length_slice).ok()?.parse::<usize>().ok()?;
    
    let value_idx = colon_idx + 1;
    let value_end_idx = value_idx + length;
    
    if value_end_idx > content.len() {
        Some(Err(ParseError::UnexpectedEndOfData))
    } else {
        let value = content[value_idx..value_end_idx].to_vec();

        Some(Ok((value, value_end_idx)))
    }
}

fn parse_list(content: &[u8]) -> Option<Result<(Vec<Node>, usize), ParseError>> {
    if content[0] != b'l' {
        return None;
    };

    let mut list = Vec::new();
    let mut parse_len = 1;

    loop {
        if parse_len == content.len() {
            return Some(Err(ParseError::UnexpectedEndOfData));
        }

        if content[parse_len] == b'e' {
            parse_len += 1;
            break;
        }

        let content_from_value = &content[parse_len..];
        let (value, value_parse_len) = match parse_value(content_from_value) {
            Err(err) => return Some(Err(err)),
            Ok(value) => value,
        };

        list.push(Node { value, unparsed: &content_from_value[..value_parse_len] });
        parse_len += value_parse_len;
    }

    Some(Ok((list, parse_len)))
}

fn parse_dictionary(content: &[u8]) -> Option<Result<(HashMap<String, Node>, usize), ParseError>> {
    if content[0] != b'd' {
        return None;
    };

    let mut dict = HashMap::new();
    let mut parse_len = 1;

    loop {
        if parse_len == content.len() {
            return Some(Err(ParseError::UnexpectedEndOfData));
        }

        if content[parse_len] == b'e' {
            parse_len += 1;
            break;
        }

        let content_from_key = &content[parse_len..];
        let (key_byte_str, key_parse_len) = 
            match parse_byte_string(content_from_key) {
                None => {
                    return Some(Err(ParseError::NonByteStringDictKey))
                },
                Some(value) => {
                    match value {
                        Err(err) => return Some(Err(err)),
                        Ok(value) => value,
                    }
                }, 
            };

        let key = match str::from_utf8(&key_byte_str) {
            Err(_err) => return Some(Err(ParseError::NonUtf8DictKey)),
            Ok(key) => key.to_string(),
        };

        let content_from_value = &content_from_key[key_parse_len..];
        let (value, value_parse_len) = match parse_value(content_from_value) {
            Err(err) => return Some(Err(err)),
            Ok(value) => value,
        };

        dict.insert(key, Node { value, unparsed: &content_from_value[..value_parse_len] });
        parse_len += key_parse_len + value_parse_len;
    } 

    Some(Ok((dict, parse_len)))
}

