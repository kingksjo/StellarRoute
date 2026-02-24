use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    Paused = 4,
    InvalidRoute = 10,
    RouteExpired = 11,
    EmptyRoute = 12,
    TooManyHops = 13,
    InsufficientInput = 20,
    InsufficientOutput = 21,
    SlippageExceeded = 22,
    DeadlineExceeded = 23,
    ExecutionTooEarly = 24,
    PriceImpactTooHigh = 25,
    SpreadTooHigh = 26,
    PoolNotSupported = 30,
    PoolCallFailed = 31,
    ReserveManipulationDetected = 32,
    InvalidAmount = 40,
    Overflow = 41,
    CommitmentRequired = 50,
    CommitmentNotFound = 51,
    CommitmentExpired = 52,
    InvalidReveal = 53,
    RateLimitExceeded = 60,
}
