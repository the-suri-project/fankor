use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::AccountInfo;
use solana_program::instruction::AccountMeta;
use solana_program::msg;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;

use crate::cpi;
use crate::cpi::system_program::CpiCreateAccount;
use crate::errors::{FankorErrorCode, FankorResult};
use crate::models::{Argument, FankorContext, Program, System, UncheckedAccount};
use crate::prelude::{byte_seeds_to_slices, FnkVec};
use crate::traits::{AccountInfoVerification, CpiInstruction, Instruction, LpiInstruction};

/// Instruction that allows to create/modify program accounts in order facilitate
/// program testing.
pub struct TestInstruction<'info> {
    /// The action to perform.
    pub args: Argument<TestInstructionAction>,

    /// The account to modify.
    /// This account must be signer if args is `TestInstructionAction::Init`.
    pub account: UncheckedAccount<'info>,

    /// The account that pays/receives the rent difference.
    pub payer: UncheckedAccount<'info>,

    /// System program to call their instructions.
    pub system_program: Program<'info, System>,
}

#[derive(Clone, Debug, Eq, PartialEq, BorshSerialize, BorshDeserialize)]
pub enum TestInstructionAction {
    /// Initializes a new account.
    Init { bytes: FnkVec<u8> },

    /// Initializes a new account with a PDA.
    InitPDA {
        seeds: FnkVec<u8>,
        bytes: FnkVec<u8>,
    },

    /// Replaces the bytes of an account.
    /// The account must be already initialized.
    Replace { bytes: FnkVec<u8> },

    /// Appends the bytes too the end of an account.
    Append { bytes: FnkVec<u8> },

    /// Closes an account.
    Close,
}

impl<'info> TestInstruction<'info> {
    // METHODS ----------------------------------------------------------------

    pub fn processor(self, context: FankorContext<'info>) -> FankorResult<()> {
        match self.args.into_inner() {
            TestInstructionAction::Init { bytes } => {
                msg!("Testable Instruction: Init");

                // Init account.
                let rent = Rent::get()?;
                let lamports = rent.minimum_balance(bytes.len());

                cpi::system_program::create_account(
                    &self.system_program,
                    CpiCreateAccount {
                        from: self.payer.info().clone(),
                        to: self.account.info().clone(),
                    },
                    lamports,
                    bytes.len() as u64,
                    context.program_id(),
                    &[],
                )?;

                // Write bytes.
                let mut data_bytes = self.account.info().try_borrow_mut_data().map_err(|_| {
                    FankorErrorCode::ZeroCopyPossibleDeadlock {
                        type_name: std::any::type_name::<Self>(),
                    }
                })?;
                let data_bytes = &mut data_bytes[..];
                data_bytes.copy_from_slice(&bytes);
            }
            TestInstructionAction::InitPDA { seeds, bytes } => {
                msg!("Testable Instruction: InitPDA");

                // Init account.
                let rent = Rent::get()?;
                let lamports = rent.minimum_balance(bytes.len());
                let seeds = byte_seeds_to_slices(seeds.as_slice());

                cpi::system_program::create_account(
                    &self.system_program,
                    CpiCreateAccount {
                        from: self.payer.info().clone(),
                        to: self.account.info().clone(),
                    },
                    lamports,
                    bytes.len() as u64,
                    context.program_id(),
                    &[&seeds],
                )?;

                // Write bytes.
                let mut data_bytes = self.account.info().try_borrow_mut_data().map_err(|_| {
                    FankorErrorCode::ZeroCopyPossibleDeadlock {
                        type_name: std::any::type_name::<Self>(),
                    }
                })?;
                let data_bytes = &mut data_bytes[..];
                data_bytes.copy_from_slice(&bytes);
            }
            TestInstructionAction::Replace { bytes } => {
                msg!("Testable Instruction: Replace");

                // Realloc account.
                self.account.realloc_unchecked(
                    bytes.len(),
                    false,
                    Some(self.payer.info()),
                    &self.system_program,
                )?;

                // Append bytes.
                let mut data_bytes = self.account.info().try_borrow_mut_data().map_err(|_| {
                    FankorErrorCode::ZeroCopyPossibleDeadlock {
                        type_name: std::any::type_name::<Self>(),
                    }
                })?;
                let data_bytes = &mut data_bytes[..];
                data_bytes.copy_from_slice(&bytes);
            }
            TestInstructionAction::Append { bytes } => {
                msg!("Testable Instruction: Append");

                // Realloc account.
                let length = self.account.info().data_len();
                self.account.realloc_unchecked(
                    length + bytes.len(),
                    false,
                    Some(self.payer.info()),
                    &self.system_program,
                )?;

                // Append bytes.
                let mut data_bytes = self.account.info().try_borrow_mut_data().map_err(|_| {
                    FankorErrorCode::ZeroCopyPossibleDeadlock {
                        type_name: std::any::type_name::<Self>(),
                    }
                })?;
                let data_bytes = &mut data_bytes[length..];
                data_bytes.copy_from_slice(&bytes);
            }
            TestInstructionAction::Close => {
                self.account.close(self.payer.info())?;
            }
        }

        Ok(())
    }
}

#[automatically_derived]
impl<'info> Instruction<'info> for TestInstruction<'info> {
    type CPI = CpiTestInstruction;
    type LPI = LpiTestInstruction<'info>;
    fn try_from(
        context: &'info FankorContext<'info>,
        buf: &mut &[u8],
        accounts: &mut &'info [AccountInfo<'info>],
    ) -> FankorResult<Self> {
        let args =
            <Argument<TestInstructionAction> as Instruction>::try_from(context, buf, accounts)?;
        let account = <UncheckedAccount<'info> as Instruction>::try_from(context, buf, accounts)?;
        let payer = <UncheckedAccount<'info> as Instruction>::try_from(context, buf, accounts)?;
        let system_program =
            <Program<'info, System> as Instruction>::try_from(context, buf, accounts)?;

        let result = Self {
            args,
            account,
            payer,
            system_program,
        };

        result.validate(context)?;

        Ok(result)
    }
}

impl<'info> TestInstruction<'info> {
    fn validate(&self, _context: &'info FankorContext<'info>) -> FankorResult<()> {
        let mut verification_config = AccountInfoVerification::default();
        let mut closure = |info: &AccountInfo<'info>| {
            if !info.is_writable {
                return Err(
                    FankorErrorCode::AccountConstraintNotWritable { account: "account" }.into(),
                );
            }

            if matches!(self.args.as_ref(), TestInstructionAction::Init { .. }) && !info.is_signer {
                return Err(
                    FankorErrorCode::AccountConstraintNotSigner { account: "account" }.into(),
                );
            }

            Ok(())
        };
        verification_config.account_info = Some(&mut closure);
        self.account
            .verify_account_infos(&mut verification_config)?;

        let mut verification_config = AccountInfoVerification::default();
        let mut closure = |info: &AccountInfo<'info>| {
            if !info.is_writable {
                return Err(
                    FankorErrorCode::AccountConstraintNotWritable { account: "payer" }.into(),
                );
            }

            if !info.is_signer {
                return Err(
                    FankorErrorCode::AccountConstraintNotSigner { account: "payer" }.into(),
                );
            }

            Ok(())
        };
        verification_config.account_info = Some(&mut closure);
        self.payer.verify_account_infos(&mut verification_config)?;

        Ok(())
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub struct CpiTestInstruction {}

impl<'info> CpiInstruction<'info> for CpiTestInstruction {
    fn serialize_into_instruction_parts<W: std::io::Write>(
        &self,
        _writer: &mut W,
        _metas: &mut Vec<AccountMeta>,
        _infos: &mut Vec<AccountInfo<'info>>,
    ) -> FankorResult<()> {
        unreachable!("CpiTestInstruction should never be used")
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub struct LpiTestInstruction<'info> {
    pub args: <Argument<TestInstructionAction> as Instruction<'info>>::LPI,
    pub account: <UncheckedAccount<'info> as Instruction<'info>>::LPI,
    pub payer: <UncheckedAccount<'info> as Instruction<'info>>::LPI,
    pub system_program: <Program<'info, System> as Instruction<'info>>::LPI,
}

impl<'info> LpiInstruction for LpiTestInstruction<'info> {
    fn serialize_into_instruction_parts<W: std::io::Write>(
        &self,
        writer: &mut W,
        metas: &mut Vec<AccountMeta>,
    ) -> FankorResult<()> {
        LpiInstruction::serialize_into_instruction_parts(&self.args, writer, metas)?;
        LpiInstruction::serialize_into_instruction_parts(&self.account, writer, metas)?;
        let mut meta = metas.last_mut().unwrap();
        meta.is_writable = true;
        meta.is_signer = matches!(self.args.as_ref(), TestInstructionAction::Init { .. });

        LpiInstruction::serialize_into_instruction_parts(&self.payer, writer, metas)?;
        meta = metas.last_mut().unwrap();
        meta.is_writable = true;
        meta.is_signer = true;

        LpiInstruction::serialize_into_instruction_parts(&self.system_program, writer, metas)?;

        Ok(())
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

/// Creates a new test instruction.
pub fn create_test_instruction<'info>(
    accounts: <TestInstruction<'info> as Instruction<'info>>::LPI,
    program_id: &Pubkey,
) -> FankorResult<solana_program::instruction::Instruction> {
    let mut data = vec![0];
    let mut metas = Vec::new();

    LpiInstruction::serialize_into_instruction_parts(&accounts, &mut data, &mut metas)?;

    Ok(solana_program::instruction::Instruction {
        program_id: *program_id,
        accounts: metas,
        data,
    })
}
