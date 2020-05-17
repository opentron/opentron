#![feature(test)]

extern crate test;
use node_cli::config::Config;
use node_cli::db::ChainDB;
use test::Bencher;

#[bench]
fn bench_get_block_1000_from_genesis(b: &mut Bencher) {
    let config = Config::load_from_file("./conf.toml").unwrap();
    let db = ChainDB::new(&config.storage.data_dir);
    println!("db opened");

    db.await_background_jobs();

    let start = 0;
    b.iter(|| {
        for i in start..start + 1000 {
            let _ = db.get_block_by_number(i as _);
        }
    });
}

#[bench]
fn bench_get_block_1000_from_tail(b: &mut Bencher) {
    let config = Config::load_from_file("./conf.toml").unwrap();
    let db = ChainDB::new(&config.storage.data_dir);
    println!("db opened");

    db.await_background_jobs();

    let start = db.get_block_height() - 1001;
    b.iter(|| {
        for i in start..start + 1000 {
            let _ = db.get_block_by_number(i as _);
        }
    });
}

#[bench]
fn bench_get_block_1000_from_middle(b: &mut Bencher) {
    let config = Config::load_from_file("./conf.toml").unwrap();
    let db = ChainDB::new(&config.storage.data_dir);
    println!("db opened");

    db.await_background_jobs();

    let start = db.get_block_height() / 2 - 500;
    b.iter(|| {
        for i in start..start + 1000 {
            let _ = db.get_block_by_number(i as _);
        }
    });
}

#[bench]
fn bench_get_block_1000_random(b: &mut Bencher) {
    let config = Config::load_from_file("./conf.toml").unwrap();
    let db = ChainDB::new(&config.storage.data_dir);
    println!("db opened");

    db.await_background_jobs();

    let end = db.get_block_height();
    let step = end / 1001;
    let target = (0..end).step_by(step as _).take(1000).collect::<Vec<_>>();
    b.iter(|| {
        for &i in &target {
            let _ = db.get_block_by_number(i as _);
        }
    });
}
