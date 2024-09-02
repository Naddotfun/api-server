pub mod change_channels {
    pub const TOKEN: &str = "new_token";
    pub const CURVE: &str = "update_curve";
    pub const SWAP: &str = "new_swap";
    pub const CHART: &str = "new_chart";
    pub const BALANCE: &str = "balance_change";
    pub const TOKEN_REPLIES_COUNT: &str = "new_token_reply";
    pub const THREAD: &str = "thread_change";
    pub const ALL: [&str; 7] = [
        TOKEN,
        CURVE,
        SWAP,
        CHART,
        BALANCE,
        THREAD,
        TOKEN_REPLIES_COUNT,
    ];
}
