extern crate asianscreens;

use asianscreens::client;

fn main() {
    println!(
        "{}",
        client::find("ren mitsuki").unwrap().unwrap().birthdate
    );
}
