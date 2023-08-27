fn main() {
    println!("Hello, world!");
}

struct Page {
    title: String,
}

impl Page {
    pub fn new() -> Page {
        Page {
            title: "".to_string(),
        }
    }
}

fn read_md_file(_path: &str) -> Page {
    Page::new()
}


#[test]
fn test_read() {
    let data = read_md_file("examples/pages/index.md");
    let expected = Page {
        title: "".to_string(),
    };
    assert_eq!(data.title, expected.title);
}