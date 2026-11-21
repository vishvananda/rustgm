//! A test's self-check holder never reaches a test running beside it.
#[test]
fn a_holder_switches_the_checks_on_only_for_its_own_thread()
-> Result<(), Box<dyn std::error::Error>> {
    let checking = super::SelfCheck::hold();
    assert!(super::self_check());
    let beside = std::thread::spawn(super::self_check)
        .join()
        .map_err(|_| "self-check probe thread failed")?;
    assert!(!beside);
    drop(checking);
    assert!(!super::self_check());
    Ok(())
}
