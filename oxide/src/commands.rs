use std::collections::HashMap;

// static commands: HashMap<&str, fn(&String)>  = vec!
// [
//     ("ls", List),
//     ("list", List),
//     ("dir", List),
//     ("cat", Display),
//     ("display", Display)
// ].into_iter().collect(); 

fn List (filepath: &String) {
    println!("In List!");
}

fn Display (filepath: &String) {
    println!("In Display!");
}