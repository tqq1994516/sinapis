use rand::rngs::OsRng;
use ed25519_dalek::{SigningKey, pkcs8::{EncodePrivateKey, EncodePublicKey, spki::der::pem::LineEnding}};

fn main() {
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();

    let project_dir = std::env::current_dir().unwrap();
    let parent_dir = project_dir.parent().unwrap();

    // 将私钥转换为PKCS#8格式并编码写入PEM
    signing_key.write_pkcs8_pem_file(parent_dir.join("private.pem"), LineEnding::LF).unwrap();
    // 将公钥转换为PKCS#8格式并编码写入PEM
    verifying_key.write_public_key_pem_file(parent_dir.join("public.pem"), LineEnding::LF).unwrap();

}
