use fankor::prelude::*;

pub use enum_account::*;
pub use struct_account::*;

mod enum_account;
mod struct_account;

#[accounts]
pub enum ProgramAccount {
    #[discriminant = 5]
    StructAccountData,
    StructAccountData2,
    ZeroCopyStructAccountData,
}

#[accounts(base = ProgramAccount)]
pub enum ProgramAccountSubSet {
    StructAccountData,
}

#[accounts(base = ProgramAccount)]
pub enum ProgramAccountZeroSubSet {
    StructAccountData,
    ZeroCopyStructAccountData,
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use crate::accounts::StructAccountData;

    #[test]
    fn test_size() {
        let value = ProgramAccount::StructAccountData(StructAccountData {
            value1: 0,
            value2: "test".to_string(),
        });

        assert_eq!(value.byte_size(), 1 + 4 + 4 + 4);
        assert_eq!(ProgramAccount::min_byte_size(), 1 + 4);
    }

    #[test]
    fn test_zc_size() {
        let mut lamports = 0;
        let mut vector = vec![5u8, 1, 0, 0, 0, 2u8, 0, 0, 0, 33, 44];
        let info = create_account_info_for_tests(&mut lamports, &mut vector);
        let zc = Zc::<ProgramAccount>::new_unchecked(&info, 0);

        assert_eq!(zc.byte_size().unwrap(), 1 + 4 + 4 + 2);

        let zc_value = zc.zc_value().unwrap();
        match zc_value {
            ZcProgramAccount::StructAccountData(v) => {
                assert_eq!(v.byte_size().unwrap(), 1 + 4 + 4 + 2);
            }
            _ => {
                panic!("Unexpected discriminant");
            }
        }
    }

    #[test]
    fn test_zc_read() {
        let mut lamports = 0;
        let mut vector = vec![5u8, 1u8, 0, 0, 0, 4u8, 0, 0, 0];
        let string = "test";
        for b in string.bytes() {
            vector.push(b);
        }

        let info = create_account_info_for_tests(&mut lamports, &mut vector);
        let zc = Zc::<ProgramAccount>::new_unchecked(&info, 0);
        let value = zc.try_value().unwrap();
        match value {
            ProgramAccount::StructAccountData(v) => {
                assert_eq!(
                    v,
                    StructAccountData {
                        value1: 1,
                        value2: "test".to_string(),
                    }
                );
            }
            _ => {
                panic!("Unexpected discriminant");
            }
        }
    }
}
