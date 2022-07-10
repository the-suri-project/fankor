use fankor::prelude::*;

#[derive(Clone, FankorSerialize, FankorDeserialize)]
pub struct AuthorityAddPaymentMethodArguments {
    pub active: bool,
}
