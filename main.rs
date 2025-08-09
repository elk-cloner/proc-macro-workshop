// use derive_builder::Builder;

// #[derive(Builder)]
// pub struct Command {
//     executable: String,
//     #[builder(eac = "arg")]
//     args: Vec<String>,
//     env: Vec<String>,
//     current_dir: Option<String>,
// }

// fn main() {}

use derive_debug::CustomDebug;

#[derive(CustomDebug)]
pub struct Field {
    name: &'static str,
    #[debug = "0b{:08b}"]
    bitmask: u8,
}

fn main() {
    // let f = Field {
    //     name: "F",
    //     bitmask: 0b00011100,
    // };

    // let debug = format!("{:?}", f);

    // assert!(debug.starts_with(r#"Field { name: "F","#));
}
