//! New convention bytes require the explicit foreign profile; old profiles stay exact.
use rustgm::support::crates::{
    self, Abi, CrateIdentity, ExternalDef, ExternalKind, Function, Interface, Scalar, Signature,
    Symbol,
};

fn artifact(abi: Abi) -> Result<Interface, &'static str> {
    let owner = CrateIdentity::new("foreign", "wire")?;
    let function = Function::new(
        ExternalDef::new(owner.clone(), vec!["f".into()], ExternalKind::Function)?,
        Signature::new(vec![Scalar::U32], Scalar::U32, true)?.with_abi(abi),
        Symbol::Exact("foreign_f".into()),
    )?;
    Interface::new(owner, vec![], vec![function])
}

#[test]
fn fixed_foreign_signatures_roundtrip_and_old_profiles_keep_their_wire_contract()
-> Result<(), Box<dyn std::error::Error>> {
    let original = artifact(Abi::C)?;
    super::public_names::roundtrip(&original)?;
    let bytes = crates::encode(&original)?;
    assert_eq!(&bytes[12..14], &[4, 0]);
    let restored = crates::decode(&bytes)?;
    assert_eq!(restored, original);
    assert_eq!(crates::encode(&restored)?, bytes);
    for end in 0..bytes.len() {
        assert!(crates::decode(&bytes[..end]).is_err(), "prefix {end}");
    }
    let old = crates::encode(&artifact(Abi::Rust)?)?;
    assert_eq!(old[12], 0);
    assert_eq!(crates::encode(&crates::decode(&old)?)?, old);
    let abi = 14 + 8 + "foreign".len() + "wire".len() + 4 + 4 + 4 + 3 + 1;
    assert_eq!(&bytes[abi..abi + 2], &[1, 0]);
    for (offset, replacement) in [(13, 2), (abi, 0), (abi, 2), (abi + 1, 1), (12, 2)] {
        let mut invalid = bytes.clone();
        invalid[offset] = replacement;
        assert!(
            crates::decode(&invalid).is_err(),
            "offset {offset}: {replacement}"
        );
    }
    let mut invalid_old = old;
    invalid_old[abi - 1] = 1;
    assert_eq!(
        crates::decode(&invalid_old).err(),
        Some("unsupported metadata calling convention")
    );
    Ok(())
}
