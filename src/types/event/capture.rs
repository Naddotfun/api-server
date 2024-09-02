use crate::types::model::{
    BalanceWrapper, ChartWrapper, Curve, Swap, Thread, ThreadWrapper, Token, TokenReplyCount,
};

#[derive(Clone, Debug)]
pub enum TokenEventCapture {
    Token(Token),
    Swap(Swap),
    Chart(ChartWrapper),
    Balance(BalanceWrapper),
    Curve(Curve),
    Thread(ThreadWrapper),
}
#[derive(Clone, Debug)]
pub enum NewContentCapture {
    NewSwap(Swap),
    NewToken(Token),
}

#[derive(Debug, Clone)]
pub enum OrderEventCapture {
    CreationTime(Token),
    BumpOrder(Swap),
    ReplyChange(TokenReplyCount),
    MartKetCap(Curve),
    // ThreadChange(Thread),
}
