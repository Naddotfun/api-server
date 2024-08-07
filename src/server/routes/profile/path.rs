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
            Self::ProfileByNickname => "/profile/:nickname",
            Self::ProfileByAddress => "/profile/:address",
            Self::CoinHeldByNickname => "/profile/coins-held/:nickname",
            Self::CoinHeldByAddress => "/profile/coins-held/:address",
            Self::RepliesByNickname => "/profile/replies/:nickname",
            Self::RepliesByAddress => "/profile/replies/:address",
            Self::CoinCreatedByNickname => "/profile/coins-created/:nickname",
            Self::CoinCreatedByAddress => "/profile/coins-created/:address",
            Self::FollowersByNickname => "/profile/followers/:nickname",
            Self::FollowersByAddress => "/profile/followers/:address",
            Self::FollowingByNickname => "/profile/following/:nickname",
            Self::FollowingByAddress => "/profile/following/:address",
        }
    }

    pub fn docs_str(&self) -> &'static str {
        match self {
            Self::ProfileByNickname => "/profile/{nickname}",
            Self::ProfileByAddress => "/profile/{address}",
            Self::CoinHeldByNickname => "/profile/coins-held/{nickname}",
            Self::CoinHeldByAddress => "/profile/coins-held/{address}",
            Self::RepliesByNickname => "/profile/replies/{nickname}",
            Self::RepliesByAddress => "/profile/replies/{address}",
            Self::CoinCreatedByNickname => "/profile/coins-created/{nickname}",
            Self::CoinCreatedByAddress => "/profile/coins-created/{address}",
            Self::FollowersByNickname => "/profile/followers/{nickname}",
            Self::FollowersByAddress => "/profile/followers/{address}",
            Self::FollowingByNickname => "/profile/following/{nickname}",
            Self::FollowingByAddress => "/profile/following/{address}",
        }
    }
}
