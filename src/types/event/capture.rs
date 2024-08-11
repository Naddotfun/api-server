use crate::types::model::{
    BalanceWrapper, ChartWrapper, Coin, CoinReplyCount, Curve, Swap, ThreadWrapper,
};

#[derive(Clone, Debug)]
pub enum CoinEventCapture {
    Coin(Coin),
    Swap(Swap),
    Chart(ChartWrapper),
    Balance(BalanceWrapper),
    Curve(Curve),
    Thread(ThreadWrapper),
}
#[derive(Clone, Debug)]
pub enum NewContentCapture {
    NewSwap(Swap),
    NewToken(Coin),
}

#[derive(Debug, Clone)]
pub enum OrderEventCapture {
    CreationTime(Coin),
    BumpOrder(Swap),
    ReplyChange(CoinReplyCount),
    MartKetCap(Curve),
}
