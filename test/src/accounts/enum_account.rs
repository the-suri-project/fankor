use fankor::prelude::*;

#[fankor_base]
#[derive(FieldOffsets)]
pub enum EnumAccountData {
    A,
    B(u32),
    C { value1: u32, value2_snake: String },
    D { value4: (), value5: FnkExtension },
}

#[derive(EnumDiscriminants, FankorZeroCopy)]
pub enum ZeroCopyEnumWithoutValues {
    A,
    B,
    C,
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
        let value = EnumAccountData::C {
            value1: 0,
            value2_snake: "test".to_string(),
        };

        assert_eq!(value.byte_size(), 1 + 4 + 4 + 4);
        assert_eq!(StructAccountData::min_byte_size(), 1 + 4 + 4);
    }

    #[test]
    fn test_zc_size() {
        let mut lamports = 0;
        let mut vector = vec![2u8, 1, 0, 0, 0, 2u8, 0, 0, 0, 33, 44];
        let info = create_account_info_for_tests(&mut lamports, &mut vector);
        let zc = Zc::<EnumAccountData>::new_unchecked(&info, 0);

        assert_eq!(zc.byte_size().unwrap(), 1 + 4 + 4 + 2);

        let zc_value = zc.zc_value().unwrap();
        match zc_value {
            ZcEnumAccountData::C {
                value1,
                value2_snake,
            } => {
                assert_eq!(value1.byte_size().unwrap(), 4);
                assert_eq!(value2_snake.byte_size().unwrap(), 4 + 2);
            }
            _ => {
                panic!("Unexpected variant");
            }
        }
    }

    #[test]
    fn test_zc_read() {
        let mut lamports = 0;
        let mut vector = vec![2u8, 1, 0, 0, 0, 4u8, 0, 0, 0];
        let string = "test";

        for b in string.bytes() {
            vector.push(b);
        }

        let vector_save = vector.clone();
        let info = create_account_info_for_tests(&mut lamports, &mut vector);
        let zc = Zc::<EnumAccountData>::new_unchecked(&info, 0);
        let zc_value = zc.zc_value().unwrap();

        match zc_value {
            ZcEnumAccountData::C {
                value1,
                value2_snake,
            } => {
                assert_eq!(value1.try_value().unwrap(), 1);
                assert_eq!(value2_snake.try_value().unwrap(), "test");
            }
            _ => {
                panic!("Unexpected variant");
            }
        }

        zc.info().try_borrow_mut_data().unwrap().fill(0);
        zc.try_write_value_unchecked(&EnumAccountData::C {
            value1: 1,
            value2_snake: "test".to_string(),
        })
        .unwrap();

        let data = info.try_borrow_data().unwrap();
        assert_eq!(*data, &vector_save);
    }
}
