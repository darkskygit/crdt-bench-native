use std::io::{Read, Write};

use y_octo::{Array, Doc, DocOptions, Map, StateVector, Text};

use crate::Crdt;
use flate2::{read::DeflateDecoder, write::DeflateEncoder, Compression};

pub struct YOctoDoc {
    doc: Doc,
    map: Map,
    list: Array,
    text: Text,
    compression: bool,
}

impl Crdt for YOctoDoc {
    type Version = StateVector;

    fn name() -> &'static str {
        "y-octo"
    }

    fn create(gc: bool, compression: bool, client_id: Option<u64>) -> Self {
        let mut options = DocOptions {
            gc,
            ..Default::default()
        };
        if client_id.is_some() {
            options.client_id = client_id.unwrap();
        }
        let doc = Doc::with_options(options);
        YOctoDoc {
            map: doc.get_or_create_map("map").unwrap(),
            list: doc.get_or_create_array("list").unwrap(),
            text: doc.get_or_create_text("text").unwrap(),
            doc,
            compression,
        }
    }

    fn text_insert(&mut self, pos: usize, text: &str) {
        self.text.insert(pos as u64, text).unwrap();
    }

    fn text_del(&mut self, pos: usize, len: usize) {
        self.text.remove(pos as u64, len as u64).unwrap()
    }

    fn get_text(&mut self) -> Box<str> {
        self.text.to_string().into_boxed_str()
    }

    fn list_insert(&mut self, pos: usize, num: i32) {
        self.list.insert(pos as u64, num).unwrap();
    }

    fn list_del(&mut self, pos: usize, len: usize) {
        self.list.remove(pos as u64, len as u64).unwrap();
    }

    fn get_list(&mut self) -> Vec<i32> {
        todo!()
    }

    fn map_insert(&mut self, key: &str, num: i32) {
        self.map.insert(key.into(), num).unwrap();
    }

    fn map_del(&mut self, key: &str) {
        self.map.remove(key);
    }

    fn get_map(&mut self) -> std::collections::HashMap<String, i32> {
        todo!()
        // let t = self.doc.transact();
        // self.map
        //     .iter(&t)
        //     .map(|(key, value)| {
        //         let v: i64 = value.to_json(&t).into();
        //         (key.to_owned(), v as i32)
        //     })
        //     .collect()
    }

    fn encode_full(&mut self) -> Vec<u8> {
        let encoded = self.doc.encode_update_v1().unwrap();

        if self.compression {
            let mut ans = vec![];
            {
                let mut c = DeflateEncoder::new(&mut ans, Compression::default());
                c.write_all(&encoded).unwrap();
                c.try_finish().unwrap();
            }
            ans
        } else {
            encoded
        }
    }

    fn decode_full(&mut self, update: &[u8]) {
        let mut ans = vec![];
        let update = if self.compression {
            let mut c = DeflateDecoder::new(update);
            c.read_to_end(&mut ans).unwrap();
            &ans
        } else {
            update
        };
        self.doc
            .apply_update_from_binary_v1(update.to_vec())
            .unwrap();
    }

    fn version(&self) -> Self::Version {
        self.doc.get_state_vector()
    }

    fn merge(&mut self, other: &mut Self) {
        let a = self.doc.get_state_vector();
        let b = other.doc.get_state_vector();
        let a_to_b = self.doc.encode_state_as_update_v1(&b).unwrap();
        let b_to_a = other.doc.encode_state_as_update_v1(&a).unwrap();
        self.doc
            .apply_update_from_binary_v1(b_to_a.to_vec())
            .unwrap();
        other
            .doc
            .apply_update_from_binary_v1(a_to_b.to_vec())
            .unwrap();
    }

    fn gc(&self) -> Result<bool, bool> {
        Ok(!self.doc.options().gc)
    }

    fn compression(&self) -> Result<bool, bool> {
        Ok(self.compression)
    }
}
