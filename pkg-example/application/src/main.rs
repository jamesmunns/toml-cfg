pub fn main() {
    println!("{}", lib_one::CONFIG.buffer_size);
    println!("{:?}", lib_one::CONFIG.choice());
    println!("{:?}", lib_one::CONFIG.other_choice);
    println!("{}", lib_two::CONFIG.greeting);
}
