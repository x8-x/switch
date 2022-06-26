use super::cache::*;

pub struct L1Cache {
    pub cache_tables: [CacheElement;1023]
}

impl L1Cache {
    // pub fn new(cache_tables: [CacheElement;1023]) -> L1Cache {
    //     // L1Cache { hash_table: "test".to_string() }
    // }

    pub fn lookup(&self) -> Option<u8> {
        None
    }
}

//     fn hash() -> u16 {

//     }

//     pub fn lookup() -> u16 {

//     }

//     pub fn insert() -> bool {

//     }

//     pub fn delete() -> bool {

//     }
