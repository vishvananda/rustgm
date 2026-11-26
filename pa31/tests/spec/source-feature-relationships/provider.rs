#[unstable(feature = "controlling_api", issue = "none")]
pub fn controlling() -> u8 { 1 }

#[unstable(feature = "dependent_api", issue = "none", implied_by = "controlling_api")]
pub fn dependent() -> u8 { 2 }

#[unstable(feature = "current_api", issue = "none", old_name = "former_api")]
pub fn renamed() -> u8 { 3 }

#[unstable(feature = "chained_api", issue = "none", implied_by = "dependent_api")]
pub fn chained() -> u8 { 4 }

#[unstable(feature = "described_api", issue = "none", old_name = "former spelling")]
pub fn described() -> u8 { 5 }
