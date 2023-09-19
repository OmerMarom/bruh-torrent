mod bencode;

fn main() {
    let list = bencode::parse("l5:world3:dfgi32ee").unwrap();
    let dict = bencode::parse("d3:key5:value4:name4:Omere").unwrap();
    
    println!("Result1: {}", list);
    println!("Result2: {}", dict);
}

