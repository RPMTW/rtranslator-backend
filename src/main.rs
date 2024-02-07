use log::error;

fn main() {
    let result = api::start();

    if let Some(err) = result.err() {
        error!("Backend Error: {err}");
    }
}
