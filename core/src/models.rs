use crate::plain;
use crate::schema::*;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use sodiumoxide::crypto::secretbox::{gen_nonce, open, seal, Key, Nonce};
use sodiumoxide::randombytes::randombytes;
use std::{hash::Hasher, io::{Cursor, Read, Write}};
use uuid::Uuid;
use twox_hash::XxHash;

#[derive(AsChangeset, Clone, Debug, Identifiable, Insertable, PartialEq, PartialOrd, Queryable)]
#[table_name = "users"]
pub struct User {
    pub id: Uuid, // internal id
    pub twitter: Vec<u8>, // twitter hash (twitter id hashed with the master key)
    pub data_version: i32,
    pub nonce: Vec<u8>,
    pub data: Vec<u8>,
}

impl User {
    //pub fn from_twitter_id(id: u64, key: &Key) ->
    pub fn new(plain: &plain::User, key: &Key) -> Self {
        let nonce = gen_nonce();

        let bytes = bincode::serialize(plain).unwrap();
        let data = seal(&bytes, &nonce, key);

        Self {
            id: Uuid::new_v4(),
            twitter: twitter_hash(plain.twitter_id, key),
            data_version: 1,
            nonce: nonce.as_ref().into(),
            data,
        }
    }

    pub fn open(&self, key: &Key) -> Option<plain::Stack<Self, plain::User>> {
        let nonce = Nonce::from_slice(&self.nonce)?;
        let bytes = open(&self.data, &nonce, key).ok()?;
        let plain = bincode::deserialize(&bytes).ok()?;
        Some(plain::Stack { cipher: self, plain })
    }
}

impl plain::Cipher for User {
    type Plain = plain::User;
    fn update(&self, plain: &plain::User, key: &Key) -> Option<Self> {
        let nonce = gen_nonce();
        let bytes = bincode::serialize(plain).ok()?;
        let data = seal(&bytes, &nonce, key);

        let mut user = self.clone();
        user.twitter = twitter_hash(plain.twitter_id, key);
        user.nonce = nonce.as_ref().into();
        user.data = data;
        Some(user)
    }
}

impl User { /*
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
    } */
}

#[derive(AsChangeset, Clone, Debug, Identifiable, Insertable, PartialEq, PartialOrd, Queryable)]
#[table_name = "reasons"]
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
    pub fn open(&self, key: &Key) -> Option<plain::Stack<Self, plain::Reason>> {
        let nonce = Nonce::from_slice(&self.nonce)?;
        let bytes = open(&self.data, &nonce, key).ok().and_then(unpad)?;
        let plain = bincode::deserialize(&bytes).ok()?;
        Some(plain::Stack { cipher: self, plain })
    }
}

impl plain::Cipher for Reason {
    type Plain = plain::Reason;
    fn update(&self, plain: &plain::Reason, key: &Key) -> Option<Self> {
        let nonce = gen_nonce();
        let bytes = bincode::serialize(plain).ok().and_then(|b| pad(b, 650))?;
        let data = seal(&bytes, &nonce, key);

        // max of 500 bytes of UTF-8
        // + 125 of overhead (atow)
        // + some future proofing
        // = pad everything to 650 bytes
        // ~ sealed as some 670ish bytes

        let mut reason = self.clone();
        reason.nonce = nonce.as_ref().into();
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

fn twitter_hash(id: u64, key: &Key) -> Vec<u8> {
    let mut hash = XxHash::default();
    for b in &key[..] { hash.write_u8(*b); }
    hash.write_u64(id);
    let mut twitter = Vec::with_capacity(8);
    twitter.write_u64::<LittleEndian>(hash.finish()).unwrap();
    twitter
}
