use toml_edit::{table, value, Document};
fn main() {
    let toml = r#"
"hello" = 'toml!' # comment 
['a'.b] 
"#;
    let mut doc = toml.parse::<Document>().expect("invalid doc");
    assert_eq!(doc.to_string(), toml);
    // let's add a new key/value pair inside a.b: c = {d = "hello"}
    doc["a"]["c"] = table();
    doc["a"]["c"]["d"] = value("hello");

    // println!("{:?}", &doc["a"]["b"]);
    println!("{:?}", &doc["a"]["e"].is_none());
    // autoformat inline table a.b.c: { d = "hello" }
    // doc["a"]["b"]["c"].as_inline_table_mut().map(|t| t.fmt());
    // doc["a"]["b"]["c"]["d"].as_table();
    let root = doc.as_table_mut();
    let servers = root.entry("a");
    let servers = servers.as_table_mut().unwrap();
    // let servers = as_table!(servers);
    assert!(servers.remove("b").is_some());

    let expected = r#"  
"hello" = 'toml!' # comment 
['a'.b] 
c = { d = "hello" }
"#;
    assert_eq!(doc.to_string(), expected);
}
