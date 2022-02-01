// Copyright (C) 2019-2022 Aleo Systems Inc.
// This file is part of the snarkOS library.

// The snarkOS library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkOS library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkOS library. If not, see <https://www.gnu.org/licenses/>.

use snarkos_storage::{storage::rocksdb::RocksDB, LedgerState};
use snarkvm::{dpc::testnet2::Testnet2, prelude::Block};

use criterion::{criterion_group, criterion_main, Criterion};

use std::{fs, time::Duration};

const NUM_BLOCKS: usize = 1_000;

fn insertion(c: &mut Criterion) {
    // Read the test blocks; note: they don't include the genesis block, as it's always available when creating a ledger.
    // note: the `blocks_100` and `blocks_1000` files were generated on a testnet2 storage using `LedgerState::dump_blocks`.
    let mut test_blocks = fs::read(format!("benches/blocks_{}", NUM_BLOCKS)).expect(&format!("Missing the test blocks file"));
    let blocks: Vec<Block<Testnet2>> = bincode::deserialize(&mut test_blocks).expect("Failed to deserialize a block dump");
    assert_eq!(blocks.len(), NUM_BLOCKS - 1);

    // Prepare a test ledger and an iterator of blocks to insert.
    let temp_dir = tempfile::tempdir().expect("Failed to open temporary directory").into_path();
    let ledger = LedgerState::open_writer::<RocksDB, _>(temp_dir, NUM_BLOCKS as u32).expect("Failed to initialize ledger");
    let mut block_iter = blocks.iter();

    c.bench_function("add_block", |b| {
        b.iter(|| {
            let next_block = if let Some(block) = block_iter.next() {
                block
            } else {
                let _ = ledger.revert_to_block_height(0);
                block_iter = blocks.iter();
                block_iter.next().unwrap()
            };
            ledger.add_next_block(next_block).expect("Failed to add a test block");
        })
    });
}

criterion_group!(
    name = benches;
    // This benchmark needs a bit more time than the default 5s.
    config = Criterion::default().measurement_time(Duration::from_secs(10));
    targets = insertion
);
criterion_main!(benches);
