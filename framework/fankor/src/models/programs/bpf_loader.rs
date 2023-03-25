use crate::traits::ProgramType;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::bpf_loader_upgradeable;
use solana_program::pubkey::Pubkey;
use std::io::{ErrorKind, Write};
use std::ops::Deref;

#[derive(Debug, Copy, Clone)]
pub struct BpfLoader;

impl ProgramType for BpfLoader {
    fn name() -> &'static str {
        "BpfLoader"
    }

    fn address() -> &'static Pubkey {
        &solana_bpf_loader_program::ID
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[derive(Debug, Copy, Clone)]
pub struct BpfLoaderUpgradeable;

impl ProgramType for BpfLoaderUpgradeable {
    fn name() -> &'static str {
        "BpfLoaderUpgradeable"
    }

    fn address() -> &'static Pubkey {
        &solana_bpf_loader_program::upgradeable::ID
    }
}

// ----------------------------------------------------------------------------
// ACCOUNTS -------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct UpgradeableLoaderAccount(bpf_loader_upgradeable::UpgradeableLoaderState);

impl crate::traits::AccountType for UpgradeableLoaderAccount {
    fn discriminant() -> u8 {
        0
    }

    fn owner() -> &'static Pubkey {
        &solana_bpf_loader_program::upgradeable::ID
    }
}

impl Default for UpgradeableLoaderAccount {
    fn default() -> Self {
        Self(bpf_loader_upgradeable::UpgradeableLoaderState::Uninitialized)
    }
}

#[cfg(any(feature = "test-utils", test))]
impl BorshSerialize for UpgradeableLoaderAccount {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let buf =
            bincode::serialize(&self.0).map_err(|e| std::io::Error::new(ErrorKind::Other, e))?;
        writer.write_all(&buf)?;

        Ok(())
    }
}

#[cfg(not(any(feature = "test-utils", test)))]
impl BorshSerialize for UpgradeableLoaderAccount {
    fn serialize<W: Write>(&self, _writer: &mut W) -> std::io::Result<()> {
        unreachable!("Cannot write accounts that does not belong to the current program")
    }
}

impl BorshDeserialize for UpgradeableLoaderAccount {
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let result = bincode::deserialize(buf)
            .map(UpgradeableLoaderAccount)
            .map_err(|e| std::io::Error::new(ErrorKind::Other, e))?;

        *buf = &[];

        Ok(result)
    }
}

impl Deref for UpgradeableLoaderAccount {
    type Target = bpf_loader_upgradeable::UpgradeableLoaderState;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
