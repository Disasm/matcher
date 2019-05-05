use std::env;
use matcher::OrderBook;
use matcher::log::VectorLogger;
use std::fs::File;
use std::io::{BufReader, BufRead};

fn main() {
    let mut args = env::args_os();
    if args.len() < 2 {
        eprintln!("Usage: matcher <filename>");
        return;
    }
    let filename = args.nth(1).unwrap();

    let f = File::open(filename).expect("invalid filename");
    let f = BufReader::new(f);

    let mut book = OrderBook::new();
    for line in f.lines() {
        let line = line.unwrap();
        //println!("{}", line);
        let order = line.parse().expect("can't parse order");
        let mut logger = VectorLogger::new();
        book.execute_order(order, &mut logger);
        for log_item in logger.as_slice() {
            println!("{}", log_item.to_string());
        }
    }
}
