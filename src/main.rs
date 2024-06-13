mod makemeahanzi;

fn main() {
    //println!("{:?}", makemeahanzi::parse_dictionary().len());
    //println!("{:?}", makemeahanzi::parse_graphics());
    let graphics = makemeahanzi::parse_graphics();
    for graphic in graphics {
        if graphic.character == '口' {
            println!("{}", serde_json::to_string(&graphic).unwrap())
        }
    }
}

