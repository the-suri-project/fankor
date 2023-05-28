use fankor::prelude::*;

use crate::accounts::ProgramAccountDiscriminant;

#[account(base = ProgramAccount)]
#[derive(Debug, PartialEq)]
pub struct StructAccountData {
    pub value1: u32,
    pub value2: String,
}

#[account(base = ProgramAccount)]
pub struct StructAccountData2 {
    pub value: String,
}

#[account(base = ProgramAccount)]
#[derive(FieldOffsets)]
pub struct ZeroCopyStructAccountData {
    pub value1: u32,
    pub value2: String,
    pub value3: Vec<u8>,
    pub value4: (),
    pub value5: FnkExtension,
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_size() {
        let value = StructAccountData {
            value1: 0,
            value2: "test".to_string(),
        };

        assert_eq!(value.byte_size(), 1 + 4 + 4 + 4);
        assert_eq!(StructAccountData::min_byte_size(), 1 + 4 + 4);
    }

    #[test]
    fn test_zc_size() {
        let mut lamports = 0;
        let mut vector = vec![5u8, 1, 0, 0, 0, 2u8, 0, 0, 0, 33, 44];
        let info = create_account_info_for_tests(&mut lamports, &mut vector);
        let zc = Zc::<StructAccountData>::new_unchecked(&info, 0);

        assert_eq!(zc.byte_size().unwrap(), 1 + 4 + 4 + 2);

        let zc_value = zc.zc_value().unwrap();
        assert_eq!(zc_value.value1().unwrap().offset(), 1);
        assert_eq!(zc_value.value2().unwrap().offset(), 1 + 4);
        assert_eq!(
            zc_value.value2().unwrap().offset(),
            zc_value
                .value2_from_previous_unchecked(
                    StructAccountDataFields::Value1,
                    zc_value.value1().unwrap().offset(),
                )
                .unwrap()
                .offset()
        );
    }

    #[test]
    fn test_zc_read_write() {
        let mut lamports = 0;
        let mut vector = vec![5u8, 1u8, 0, 0, 0, 4u8, 0, 0, 0];
        let string = "test";

        for b in string.bytes() {
            vector.push(b);
        }

        let vector_save = vector.clone();
        let info = create_account_info_for_tests(&mut lamports, &mut vector);
        let zc = Zc::<StructAccountData>::new_unchecked(&info, 0);
        let zc_value = zc.zc_value().unwrap();
        assert_eq!(zc_value.value1().unwrap().try_value().unwrap(), 1);
        assert_eq!(zc_value.value2().unwrap().try_value().unwrap(), "test");

        zc.info().try_borrow_mut_data().unwrap().fill(0);
        zc.try_write_value_unchecked(&StructAccountData {
            value1: 1,
            value2: "test".to_string(),
        })
            .unwrap();

        let data = info.try_borrow_data().unwrap();
        assert_eq!(*data, &vector_save);
    }
}
