use std::env;

fn main() {
    let path = env::current_dir()
        .unwrap()
        .join("logs/nimble144zdgs6pefudmqdj7zvfel0fe8ukd30k62jy4x.txt");
    //nimble144zdgs6pefudmqdj7zvfel0fe8ukd30k62jy4x.txt
    let address = path.file_stem().unwrap().to_string_lossy().to_string();

    println!("{:?}", address)
}
