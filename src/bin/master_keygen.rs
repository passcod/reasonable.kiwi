fn main() {
    sodiumoxide::init().expect("Sodium failed to init");

    let arch_public =
        std::env::args().last()
            .and_then(|b64| base64::decode(&b64).ok())
            .and_then(|bin| sodiumoxide::crypto::box_::PublicKey::from_slice(&bin))
            .expect("Missing arch public key argument");

    let key = sodiumoxide::crypto::secretbox::gen_key();
    let enc = sodiumoxide::crypto::sealedbox::seal(&key[..], &arch_public);
    println!("MASTER_KEY={}", base64::encode(&enc[..]));
}
