pub mod change_channels {
    pub const COIN: &str = "new_coin";
    pub const CURVE: &str = "update_curve";
    pub const SWAP: &str = "new_swap";
    pub const CHART: &str = "new_chart";
    pub const BALANCE: &str = "balance_change";
    pub const COIN_REPLIES_COUNT: &str = "new_coin_reply";
    pub const THREAD: &str = "thread_change";
    pub const ALL: [&str; 7] = [
        COIN,
        CURVE,
        SWAP,
        CHART,
        BALANCE,
        THREAD,
        COIN_REPLIES_COUNT,
    ];
}
