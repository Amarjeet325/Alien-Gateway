use soroban_sdk::{contracttype, Address, Vec};

/// Represents a user's collateral position across all assets.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Position {
    /// The owner of this position.
    pub user: Address,
    /// All assets deposited by this user.
    pub assets: Vec<Address>,
    /// Corresponding balances for each asset (parallel to `assets`).
    pub balances: Vec<i128>,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum DataKey {
    Admin,
    Paused,
    SupportedAsset(Address),
    Position(Address, Address), // (user, asset)
    PositionIndex,
    /// Tracks which assets a user has ever deposited into (used to build Position).
    UserAssets(Address),
}
