//! A set of constant values used in the kintsugi runtime.

/// Money matters.
pub mod currency {
    use primitives::TokenSymbol;
    pub use primitives::{Balance, CurrencyId, CurrencyId::Token, DOT, IBTC, INTR};

    pub const NATIVE_TOKEN_ID: TokenSymbol = INTR;
    pub const NATIVE_CURRENCY_ID: CurrencyId = Token(NATIVE_TOKEN_ID);
    pub const PARENT_CURRENCY_ID: CurrencyId = Token(DOT);
    pub const WRAPPED_CURRENCY_ID: CurrencyId = Token(IBTC);

    // https://github.com/paritytech/polkadot/blob/c4ee9d463adccfa3bf436433e3e26d0de5a4abbc/runtime/polkadot/src/constants.rs#L18
    pub const UNITS: Balance = NATIVE_TOKEN_ID.one();
    pub const DOLLARS: Balance = UNITS; // 10_000_000_000
    pub const CENTS: Balance = DOLLARS / 100; // 100_000_000
    pub const MILLICENTS: Balance = CENTS / 1_000; // 100_000

    pub const fn deposit(items: u32, bytes: u32) -> Balance {
        items as Balance * 20 * DOLLARS + (bytes as Balance) * 100 * MILLICENTS
    }
}

/// Time and blocks.
pub mod time {
    use btc_relay::TARGET_SPACING;
    use primitives::{BlockNumber, Moment};

    // The relay chain is limited to 12s to include parachain blocks.
    pub const MILLISECS_PER_BLOCK: u64 = 12000;

    pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;

    // These time units are defined in number of blocks.
    pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
    pub const HOURS: BlockNumber = MINUTES * 60;
    pub const DAYS: BlockNumber = HOURS * 24;
    pub const WEEKS: BlockNumber = DAYS * 7;
    pub const YEARS: BlockNumber = DAYS * 365;

    pub const BITCOIN_SPACING_MS: u32 = TARGET_SPACING * 1000;
    pub const BITCOIN_BLOCK_SPACING: BlockNumber = BITCOIN_SPACING_MS / MILLISECS_PER_BLOCK as BlockNumber;
}
