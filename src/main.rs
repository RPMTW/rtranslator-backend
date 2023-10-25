fn main() {
    let result = api::start();

    if let Some(err) = result.err() {
        println!("Backend Error: {err}");
    }
}
