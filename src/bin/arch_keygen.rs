fn main() {
    sodiumoxide::init().expect("Sodium failed to init");

    let pair = sodiumoxide::crypto::box_::gen_keypair();
    println!("ARCH_KEYS={}", base64::encode(&bincode::serialize(&pair).unwrap()));
    eprintln!("Arch public: {}", base64::encode(&pair.0[..]));
}
