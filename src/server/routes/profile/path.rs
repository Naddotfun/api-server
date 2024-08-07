#[derive(Debug)]
pub enum ProfilePath {
    ProfileByNickname,
    ProfileByAddress,
    CoinHeldByNickname,
    CoinHeldByAddress,
    RepliesByNickname,
    RepliesByAddress,
    CoinCreatedByNickname,
    CoinCreatedByAddress,
    FollowersByNickname,
    FollowersByAddress,
    FollowingByNickname,
    FollowingByAddress,
}

impl ProfilePath {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ProfileByNickname => "/profile/nickname/:nickname",
            Self::ProfileByAddress => "/profile/address/:address",
            Self::CoinHeldByNickname => "/profile/nickname/:nickname/coins-held",
            Self::CoinHeldByAddress => "/profile/address/:address/coins-held",
            Self::RepliesByNickname => "/profile/nickname/:nickname/replies",
            Self::RepliesByAddress => "/profile/address/:address/replies",
            Self::CoinCreatedByNickname => "/profile/nickname/:nickname/coins-created",
            Self::CoinCreatedByAddress => "/profile/address/:address/coins-created",
            Self::FollowersByNickname => "/profile/nickname/:nickname/followers",
            Self::FollowersByAddress => "/profile/address/:address/followers",
            Self::FollowingByNickname => "/profile/nickname/:nickname/following",
            Self::FollowingByAddress => "/profile/address/:address/following",
        }
    }

    pub fn docs_str(&self) -> &'static str {
        match self {
            Self::ProfileByNickname => "/profile/nickname/{nickname}",
            Self::ProfileByAddress => "/profile/address/{address}",
            Self::CoinHeldByNickname => "/profile/nickname/{nickname}/coins-held",
            Self::CoinHeldByAddress => "/profile/address/{address}/coins-held",
            Self::RepliesByNickname => "/profile/nickname/{nickname}/replies",
            Self::RepliesByAddress => "/profile/address/{address}/replies",
            Self::CoinCreatedByNickname => "/profile/nickname/{nickname}/coins-created",
            Self::CoinCreatedByAddress => "/profile/address/{address}/coins-created",
            Self::FollowersByNickname => "/profile/nickname/{nickname}/followers",
            Self::FollowersByAddress => "/profile/address/{address}/followers",
            Self::FollowingByNickname => "/profile/nickname/{nickname}/following",
            Self::FollowingByAddress => "/profile/address/{address}/following",
        }
    }
}
