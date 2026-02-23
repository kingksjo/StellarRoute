use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)] // <--- ADD THIS LINE
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
    PoolNotSupported = 30,
    PoolCallFailed = 31,
    InvalidAmount = 40,
    Overflow = 41,
}
