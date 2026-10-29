//! Explicit finite contract limits are driver inputs, never source permissions.
use rustgm::expand::config::{Config, SourceContractLimits};

#[derive(Default)]
pub(super) struct Options {
    seen: u8,
}
impl Options {
    pub(super) fn consume(
        &mut self,
        argument: &str,
        args: &mut impl Iterator<Item = String>,
        config: &mut Config,
    ) -> Result<bool, String> {
        let (flag, inline) = argument
            .split_once('=')
            .map_or((argument, None), |(key, value)| (key, Some(value)));
        let bit = match flag {
            "--source-contract-work-limit" => 1,
            "--source-contract-attempted-bytes" => 2,
            "--source-contract-retained-bytes" => 4,
            _ => return Ok(false),
        };
        if self.seen & bit != 0 {
            return Err(format!("duplicate {flag}"));
        }
        let owned;
        let value = if let Some(value) = inline {
            value
        } else {
            owned = args
                .next()
                .ok_or_else(|| format!("{flag} requires a decimal u32 limit"))?;
            &owned
        };
        if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(format!("{flag} requires a decimal u32 limit"));
        }
        let value = value
            .parse::<u32>()
            .map_err(|_| format!("{flag} exceeds the finite u32 limit"))?;
        let old = config.source_contract_limits();
        config.set_source_contract_limits(SourceContractLimits::new(
            if bit == 1 { value } else { old.work_limit() },
            if bit == 2 {
                value
            } else {
                old.attempted_bytes()
            },
            if bit == 4 {
                value
            } else {
                old.retained_bytes()
            },
        ));
        self.seen |= bit;
        Ok(true)
    }
}
