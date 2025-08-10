use derive_debug::CustomDebug;
use std::fmt::Debug;
use std::marker::PhantomData;

type S = String;

#[derive(CustomDebug)]
pub struct Field<T> {
    marker: PhantomData<T>,
    string: S,
    #[debug = "0b{:08b}"]
    bitmask: u8,
}

fn assert_debug<F: Debug>() {}

fn main() {
    // let f = Field {
    //     name: "F",
    //     bitmask: 0b00011100,
    // };

    // let debug = format!("{:?}", f);

    // assert!(debug.starts_with(r#"Field { name: "F","#));
}
