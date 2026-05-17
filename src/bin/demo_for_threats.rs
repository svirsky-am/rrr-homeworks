
use broken_app::{concurrency};

fn main() {

    let total = concurrency::race_increment(1_000, 4);
    println!("total: {:?}", &total);
    concurrency::read_after_sleep();
    concurrency::reset_counter();
    println!("total: {:?}", &total);
    
}
