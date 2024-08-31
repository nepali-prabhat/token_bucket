use std::time::Duration;

mod token_bucket;

fn main(){
    println!("practising threads and datastructures");
    let tb = token_bucket::TokenBucket::new(100, 25, 3);
    std::thread::sleep(Duration::new(2, 0));
    println!("{:?}", tb);
}
