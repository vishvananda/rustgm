#[deprecated]fn old(){}
#[expect(deprecated)]fn checked(){fn nested(){old();}}
