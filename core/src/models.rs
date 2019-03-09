use crate::plain;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use sodiumoxide::crypto::secretbox::{gen_nonce, open, seal, Key, Nonce};
use sodiumoxide::randombytes::randombytes;
use std::io::{Cursor, Read, Write};
use uuid::Uuid;

#[derive(Clone, Debug, Queryable)]
pub struct User {
    pub id: Uuid, // internal id
    pub twitter_id: String,
    pub email: Option<String>,
    list_id: Option<String>, // key list id cache
    access_nonce: Option<Vec<u8>>, // access keys nonce
    access_keys: Option<Vec<u8>>, // access keys (bincoded)
    list_nonce: Option<Vec<u8>>, // key list nonce
}

impl Default for User {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            twitter_id: "".into(),
            email: None,
            access_nonce: None,
            access_keys: None,
            list_id: None,
            list_nonce: None,
        }
    }
}

impl User {
    pub fn decrypt_access(&self, master_key: &Key) -> Option<egg_mode::KeyPair> {
        let nonce = match self.access_nonce {
            None => return None,
            Some(ref n) => Nonce::from_slice(n)?
        };

        let keys = match self.access_keys {
            None => return None,
            Some(ref k) => open(k, &nonce, master_key).ok()?
        };

        let (key, secret): (String, String) = bincode::deserialize(&keys).ok()?;
        Some(egg_mode::KeyPair::new(key, secret))
    }

    pub fn get_list_nonce(&self) -> Option<Nonce> {
        match self.list_nonce {
            None => None,
            Some(ref n) => Nonce::from_slice(n),
        }
    }

    pub fn get_list_id(&self) -> Option<egg_mode::list::ListID> {
        match self.list_id {
            None => None,
            Some(ref id) => id.parse().map(egg_mode::list::ListID::from_id).ok()
        }
    }
}

#[derive(Clone, Debug, Queryable)]
pub struct Reason {
    id: Uuid, // internal id
    user_id: Uuid,
    nonce: Vec<u8>, // encryption nonce
    data: Vec<u8>,  // encrypted data
}

impl Default for Reason {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id: Uuid::default(),
            nonce: Vec::new(),
            data: Vec::new(),
        }
    }
}

impl Reason {
    pub fn open(&self, key: &Key) -> Option<plain::Reason> {
        let nonce = Nonce::from_slice(&self.nonce)?;
        let bytes = open(&self.data, &nonce, key).ok().and_then(unpad)?;
        let json = std::str::from_utf8(&bytes).ok()?;
        serde_json::from_str(json).ok()
    }

    pub fn update(&self, key: &Key, data: &plain::Reason) -> Option<Self> {
        let nonce = gen_nonce();
        let bytes = serde_json::to_vec(data).ok().and_then(|b| pad(b, 650))?;
        let data = seal(&bytes, &nonce, key);

        // max of 500 bytes of UTF-8
        // + 125 of overhead (atow)
        // + some future proofing
        // = pad everything to 650 bytes
        // ~ sealed as some 670ish bytes

        let mut reason = self.clone();
        reason.nonce = (&nonce[..]).into();
        reason.data = data;
        Some(reason)
    }
}

fn pad(input: Vec<u8>, max: usize) -> Option<Vec<u8>> {
    let size = input.len();
    if size > max { return None; }

    let mut buf = Cursor::new(randombytes(5 + max));
    buf.write_all(&[1]).ok()?; // padding version
    buf.write_u32::<LittleEndian>(size as u32).ok()?;
    buf.write_all(&input).ok()?;
    Some(buf.into_inner())
}

fn unpad(input: Vec<u8>) -> Option<Vec<u8>> {
    let len = input.len();
    if len < 5 { return None; }

    let mut input = Cursor::new(input);
    if input.read_u8().ok()? != 1 { return None; }

    let size = input.read_u32::<LittleEndian>().ok()? as usize;
    if len < size + 5 { return None; }

    let mut buf = Vec::with_capacity(size);
    input.take(size as u64).read_to_end(&mut buf).ok()?;
    Some(buf)
}
