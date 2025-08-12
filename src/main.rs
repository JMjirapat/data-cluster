use data_cluster::hashing::Hashing;
use data_cluster::storage::{Storage, StorageCmd, handle_storage_command};

fn main() {
    let mut ring = Hashing::<Storage>::new(3);

    for i in 0..3 {
        let storage = Storage::new(format!("storage_{}", i));
        ring.add_node(&storage);
    }

    let storage_cmds = vec![
        StorageCmd::Set {
            key: "key1".to_string(),
            value: "value1".to_string(),
        },
        StorageCmd::Set {
            key: "key2".to_string(),
            value: "value2".to_string(),
        },
        StorageCmd::Get {
            key: "key1".to_string(),
        },
    ];

    for cmd in storage_cmds {
        match cmd {
            StorageCmd::Set { key, value } => {
                if let Some(storage) = ring.get(&key) {
                    let response = handle_storage_command(StorageCmd::Set { key, value }, storage);
                    println!("{:?}", response);
                }
            }
            StorageCmd::Get { key } => {
                if let Some(storage) = ring.get(&key) {
                    let response = handle_storage_command(StorageCmd::Get { key }, storage);
                    println!("{:?}", response);
                }
            }
            _ => {}
        }
    }
}
