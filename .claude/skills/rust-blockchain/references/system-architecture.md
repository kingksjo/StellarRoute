# Blockchain System Architecture

## Table of Contents
- [Architecture Principles](#architecture-principles)
- [Layered Architecture](#layered-architecture)
- [Microservices Patterns](#microservices-patterns)
- [Data Architecture](#data-architecture)
- [Network Architecture](#network-architecture)
- [Scalability Patterns](#scalability-patterns)
- [Interoperability](#interoperability)
- [Production Architecture](#production-architecture)

## Architecture Principles

### Core Principles

#### 1. Separation of Concerns
Divide system into distinct modules with clear responsibilities:

```rust
// ✅ Good: Separated concerns
mod storage {
    // Only handles data persistence
    pub fn save(key: &str, value: &[u8]) -> Result<()> { }
    pub fn load(key: &str) -> Result<Vec<u8>> { }
}

mod validation {
    // Only handles validation logic
    pub fn validate_transaction(tx: &Transaction) -> Result<()> { }
}

mod business_logic {
    // Coordinates storage and validation
    pub fn process_transaction(tx: Transaction) -> Result<()> {
        validation::validate_transaction(&tx)?;
        storage::save(&tx.id, &tx.data)?;
        Ok(())
    }
}
```

#### 2. Immutability
Prefer immutable data structures:

```rust
// ✅ Good: Immutable state transitions
#[derive(Clone)]
pub struct State {
    balance: u64,
    nonce: u64,
}

impl State {
    pub fn with_balance(self, balance: u64) -> Self {
        Self { balance, ..self }
    }
    
    pub fn with_nonce(self, nonce: u64) -> Self {
        Self { nonce, ..self }
    }
}

// Usage creates new state instead of mutating
let new_state = old_state
    .with_balance(100)
    .with_nonce(old_state.nonce + 1);
```

#### 3. Fail-Fast
Detect errors early in the process:

```rust
pub fn process_transfer(from: &Account, to: &Account, amount: u64) -> Result<()> {
    // ✅ Validate everything first
    require!(amount > 0, Error::InvalidAmount);
    require!(from.balance >= amount, Error::InsufficientBalance);
    require!(from != to, Error::SelfTransfer);
    require!(is_account_active(from), Error::InactiveAccount);
    require!(is_account_active(to), Error::InactiveAccount);
    
    // Only proceed if all validations pass
    execute_transfer(from, to, amount)
}
```

## Layered Architecture

### Three-Tier Architecture

```
┌─────────────────────────────────────┐
│      Application Layer              │  User-facing logic
│  - RPC handlers                     │  - Transaction construction
│  - Event subscriptions              │  - Query interfaces
│  - CLI/UI interfaces                │
├─────────────────────────────────────┤
│      Business Logic Layer           │  Core blockchain logic
│  - Smart contracts/Pallets          │  - State transitions
│  - Transaction validation           │  - Business rules
│  - Consensus participation          │
├─────────────────────────────────────┤
│      Data Layer                     │  State management
│  - Storage abstractions             │  - Database access
│  - State queries                    │  - Persistence
│  - Merkle trees                     │
└─────────────────────────────────────┘
```

### Implementation Example

```rust
// Data Layer: Storage abstraction
pub trait Storage {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>>;
    fn set(&mut self, key: &[u8], value: Vec<u8>);
    fn delete(&mut self, key: &[u8]);
}

// Business Logic Layer: Account management
pub struct AccountManager<S: Storage> {
    storage: S,
}

impl<S: Storage> AccountManager<S> {
    pub fn get_balance(&self, account: &AccountId) -> Balance {
        self.storage
            .get(&Self::balance_key(account))
            .and_then(|bytes| Balance::decode(&bytes).ok())
            .unwrap_or_default()
    }
    
    pub fn transfer(
        &mut self,
        from: &AccountId,
        to: &AccountId,
        amount: Balance,
    ) -> Result<()> {
        // Business logic
        let from_balance = self.get_balance(from);
        ensure!(from_balance >= amount, Error::InsufficientFunds);
        
        let to_balance = self.get_balance(to);
        let new_to_balance = to_balance.checked_add(amount)
            .ok_or(Error::Overflow)?;
        
        // Update storage
        self.set_balance(from, from_balance - amount);
        self.set_balance(to, new_to_balance);
        
        Ok(())
    }
    
    fn set_balance(&mut self, account: &AccountId, balance: Balance) {
        self.storage.set(&Self::balance_key(account), balance.encode());
    }
    
    fn balance_key(account: &AccountId) -> Vec<u8> {
        [b"balance:", account.as_ref()].concat()
    }
}

// Application Layer: RPC handler
pub struct RpcHandler<S: Storage> {
    account_manager: AccountManager<S>,
}

impl<S: Storage> RpcHandler<S> {
    pub fn handle_transfer_request(
        &mut self,
        from: AccountId,
        to: AccountId,
        amount: Balance,
    ) -> RpcResponse {
        match self.account_manager.transfer(&from, &to, amount) {
            Ok(()) => RpcResponse::success(),
            Err(e) => RpcResponse::error(e.to_string()),
        }
    }
}
```

## Microservices Patterns

### Event-Driven Architecture

```rust
// Event definitions
#[derive(Debug, Clone, Encode, Decode)]
pub enum BlockchainEvent {
    TransactionSubmitted { tx_id: Hash, sender: AccountId },
    BlockProduced { block_number: u64, hash: Hash },
    StateChanged { account: AccountId, old: State, new: State },
}

// Event bus
pub trait EventBus: Send + Sync {
    fn publish(&self, event: BlockchainEvent);
    fn subscribe<F>(&self, handler: F) 
    where 
        F: Fn(BlockchainEvent) + Send + Sync + 'static;
}

// Service that produces events
pub struct TransactionPool<E: EventBus> {
    events: Arc<E>,
    pending: Mutex<Vec<Transaction>>,
}

impl<E: EventBus> TransactionPool<E> {
    pub fn submit_transaction(&self, tx: Transaction) -> Result<()> {
        // Validate
        validate_transaction(&tx)?;
        
        // Add to pool
        self.pending.lock().unwrap().push(tx.clone());
        
        // Emit event
        self.events.publish(BlockchainEvent::TransactionSubmitted {
            tx_id: tx.hash(),
            sender: tx.sender(),
        });
        
        Ok(())
    }
}

// Service that consumes events
pub struct BlockProducer<E: EventBus> {
    events: Arc<E>,
}

impl<E: EventBus> BlockProducer<E> {
    pub fn new(events: Arc<E>) -> Self {
        let producer = Self { events: events.clone() };
        
        // Subscribe to transaction events
        events.subscribe(move |event| {
            if let BlockchainEvent::TransactionSubmitted { .. } = event {
                // Trigger block production logic
            }
        });
        
        producer
    }
}
```

### Service Mesh Pattern

```rust
// Service registry for service discovery
pub struct ServiceRegistry {
    services: HashMap<String, Vec<ServiceEndpoint>>,
}

#[derive(Clone)]
pub struct ServiceEndpoint {
    pub id: String,
    pub address: SocketAddr,
    pub health: Arc<AtomicBool>,
}

impl ServiceRegistry {
    pub fn register(&mut self, service_name: String, endpoint: ServiceEndpoint) {
        self.services
            .entry(service_name)
            .or_default()
            .push(endpoint);
    }
    
    pub fn discover(&self, service_name: &str) -> Option<&ServiceEndpoint> {
        self.services
            .get(service_name)?
            .iter()
            .find(|e| e.health.load(Ordering::Relaxed))
    }
}

// Load balancer
pub struct LoadBalancer {
    registry: Arc<RwLock<ServiceRegistry>>,
}

impl LoadBalancer {
    pub fn route_request(&self, service: &str, request: Request) -> Result<Response> {
        let registry = self.registry.read().unwrap();
        let endpoint = registry.discover(service)
            .ok_or(Error::ServiceUnavailable)?;
        
        send_request(endpoint.address, request)
    }
}
```

## Data Architecture

### State Management

```rust
// Versioned state with rollback capability
pub struct VersionedState {
    current: State,
    history: Vec<(BlockNumber, State)>,
    max_history: usize,
}

impl VersionedState {
    pub fn commit(&mut self, block: BlockNumber, state: State) {
        self.history.push((block, self.current.clone()));
        
        // Prune old history
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
        
        self.current = state;
    }
    
    pub fn rollback_to(&mut self, block: BlockNumber) -> Result<()> {
        let pos = self.history
            .iter()
            .position(|(b, _)| *b == block)
            .ok_or(Error::BlockNotFound)?;
        
        self.current = self.history[pos].1.clone();
        self.history.truncate(pos);
        
        Ok(())
    }
}
```

### Merkle Tree for State Proofs

```rust
use sha2::{Sha256, Digest};

pub struct MerkleTree {
    leaves: Vec<Hash>,
    nodes: Vec<Vec<Hash>>,
}

impl MerkleTree {
    pub fn new(data: Vec<Vec<u8>>) -> Self {
        let leaves: Vec<Hash> = data.iter()
            .map(|d| hash_leaf(d))
            .collect();
        
        let mut nodes = vec![leaves.clone()];
        let mut current_level = leaves;
        
        while current_level.len() > 1 {
            current_level = Self::build_level(&current_level);
            nodes.push(current_level.clone());
        }
        
        Self { leaves, nodes }
    }
    
    pub fn root(&self) -> Hash {
        self.nodes.last()
            .and_then(|level| level.first())
            .cloned()
            .unwrap_or_default()
    }
    
    pub fn proof(&self, index: usize) -> Vec<Hash> {
        let mut proof = Vec::new();
        let mut idx = index;
        
        for level in &self.nodes[..self.nodes.len()-1] {
            let sibling_idx = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
            
            if sibling_idx < level.len() {
                proof.push(level[sibling_idx]);
            }
            
            idx /= 2;
        }
        
        proof
    }
    
    pub fn verify_proof(
        leaf: &Hash,
        proof: &[Hash],
        root: &Hash,
        index: usize,
    ) -> bool {
        let mut computed = *leaf;
        let mut idx = index;
        
        for sibling in proof {
            computed = if idx % 2 == 0 {
                hash_pair(&computed, sibling)
            } else {
                hash_pair(sibling, &computed)
            };
            idx /= 2;
        }
        
        computed == *root
    }
    
    fn build_level(current: &[Hash]) -> Vec<Hash> {
        current
            .chunks(2)
            .map(|chunk| {
                if chunk.len() == 2 {
                    hash_pair(&chunk[0], &chunk[1])
                } else {
                    chunk[0]
                }
            })
            .collect()
    }
}

fn hash_leaf(data: &[u8]) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(b"leaf:");
    hasher.update(data);
    Hash::from_slice(&hasher.finalize())
}

fn hash_pair(left: &Hash, right: &Hash) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(b"node:");
    hasher.update(left.as_ref());
    hasher.update(right.as_ref());
    Hash::from_slice(&hasher.finalize())
}
```

## Network Architecture

### P2P Network Layer

```rust
use libp2p::{
    identity, PeerId, Swarm,
    kad::{Kademlia, KademliaConfig},
    gossipsub::{Gossipsub, GossipsubConfig},
};

pub struct P2PNetwork {
    swarm: Swarm<NetworkBehaviour>,
    local_peer_id: PeerId,
}

#[derive(NetworkBehaviour)]
struct NetworkBehaviour {
    kademlia: Kademlia<MemoryStore>,
    gossipsub: Gossipsub,
}

impl P2PNetwork {
    pub fn new() -> Result<Self> {
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());
        
        let transport = libp2p::tokio_development_transport(local_key.clone())?;
        
        let behaviour = NetworkBehaviour {
            kademlia: Kademlia::new(local_peer_id, MemoryStore::new(local_peer_id)),
            gossipsub: Gossipsub::new(
                MessageAuthenticity::Signed(local_key),
                GossipsubConfig::default(),
            )?,
        };
        
        let swarm = Swarm::new(transport, behaviour, local_peer_id);
        
        Ok(Self { swarm, local_peer_id })
    }
    
    pub async fn broadcast_transaction(&mut self, tx: Transaction) -> Result<()> {
        let topic = IdentTopic::new("transactions");
        let message = tx.encode();
        
        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(topic, message)?;
        
        Ok(())
    }
    
    pub async fn discover_peers(&mut self) -> Result<Vec<PeerId>> {
        // Use Kademlia DHT for peer discovery
        let query_id = self.swarm
            .behaviour_mut()
            .kademlia
            .get_closest_peers(self.local_peer_id);
        
        // Wait for results...
        Ok(vec![])
    }
}
```

### Message Protocol

```rust
#[derive(Debug, Clone, Encode, Decode)]
pub enum NetworkMessage {
    Transaction(Transaction),
    Block(Block),
    StateRequest { block_hash: Hash },
    StateResponse { state: State },
    SyncRequest { from_block: u64, to_block: u64 },
    SyncResponse { blocks: Vec<Block> },
}

pub struct MessageHandler {
    tx_pool: Arc<TransactionPool>,
    block_store: Arc<BlockStore>,
}

impl MessageHandler {
    pub async fn handle(&self, peer: PeerId, msg: NetworkMessage) -> Result<()> {
        match msg {
            NetworkMessage::Transaction(tx) => {
                self.tx_pool.submit(tx).await?;
            }
            NetworkMessage::Block(block) => {
                self.block_store.import_block(block).await?;
            }
            NetworkMessage::StateRequest { block_hash } => {
                let state = self.block_store.get_state(&block_hash)?;
                self.send_to_peer(peer, NetworkMessage::StateResponse { state }).await?;
            }
            NetworkMessage::SyncRequest { from_block, to_block } => {
                let blocks = self.block_store.get_range(from_block..=to_block)?;
                self.send_to_peer(peer, NetworkMessage::SyncResponse { blocks }).await?;
            }
            _ => {}
        }
        Ok(())
    }
}
```

## Scalability Patterns

### Sharding

```rust
pub struct ShardedBlockchain {
    shards: Vec<Shard>,
    coordinator: Coordinator,
}

pub struct Shard {
    id: ShardId,
    state: State,
    transactions: TransactionPool,
}

impl ShardedBlockchain {
    pub fn route_transaction(&self, tx: &Transaction) -> ShardId {
        // Route based on account address
        let account_hash = hash(tx.sender().as_ref());
        ShardId(account_hash.as_u64() % self.shards.len() as u64)
    }
    
    pub async fn execute_cross_shard_tx(
        &mut self,
        tx: CrossShardTransaction,
    ) -> Result<()> {
        // Two-phase commit for cross-shard transactions
        let source_shard = self.route_transaction(&tx.source);
        let dest_shard = self.route_transaction(&tx.dest);
        
        // Phase 1: Prepare
        self.shards[source_shard as usize].prepare_send(&tx)?;
        self.shards[dest_shard as usize].prepare_receive(&tx)?;
        
        // Phase 2: Commit
        self.shards[source_shard as usize].commit_send(&tx)?;
        self.shards[dest_shard as usize].commit_receive(&tx)?;
        
        Ok(())
    }
}
```

### Layer 2 Solutions

```rust
// State channel implementation
pub struct StateChannel {
    participants: Vec<AccountId>,
    initial_state: State,
    current_state: State,
    nonce: u64,
    signatures: Vec<Signature>,
}

impl StateChannel {
    pub fn new(participants: Vec<AccountId>, initial_deposit: Balance) -> Self {
        let initial_state = State::new(initial_deposit);
        
        Self {
            participants,
            initial_state: initial_state.clone(),
            current_state: initial_state,
            nonce: 0,
            signatures: vec![],
        }
    }
    
    pub fn update_state(&mut self, new_state: State, signatures: Vec<Signature>) -> Result<()> {
        // Verify all participants signed
        ensure!(signatures.len() == self.participants.len(), Error::InsufficientSignatures);
        
        for (participant, sig) in self.participants.iter().zip(signatures.iter()) {
            let message = self.state_update_message(&new_state);
            ensure!(verify_signature(&message, sig, participant), Error::InvalidSignature);
        }
        
        self.current_state = new_state;
        self.nonce += 1;
        self.signatures = signatures;
        
        Ok(())
    }
    
    pub fn close_channel(&self) -> ChannelCloseTransaction {
        // Submit final state to mainchain
        ChannelCloseTransaction {
            channel_id: self.id(),
            final_state: self.current_state.clone(),
            nonce: self.nonce,
            signatures: self.signatures.clone(),
        }
    }
    
    fn state_update_message(&self, state: &State) -> Vec<u8> {
        (self.id(), self.nonce, state).encode()
    }
}
```

## Interoperability

### Cross-Chain Bridge

```rust
pub struct Bridge {
    source_chain: ChainId,
    dest_chain: ChainId,
    validators: Vec<AccountId>,
    threshold: u32,
}

#[derive(Encode, Decode)]
pub struct BridgeTransaction {
    pub source_tx_hash: Hash,
    pub source_block: u64,
    pub sender: AccountId,
    pub recipient: AccountId,
    pub amount: Balance,
    pub signatures: Vec<Signature>,
}

impl Bridge {
    pub fn initiate_transfer(
        &self,
        sender: &AccountId,
        recipient: &AccountId,
        amount: Balance,
    ) -> Result<BridgeTransaction> {
        // Lock tokens on source chain
        lock_tokens(sender, amount)?;
        
        // Create bridge transaction
        let tx = BridgeTransaction {
            source_tx_hash: Hash::random(),
            source_block: current_block_number(),
            sender: sender.clone(),
            recipient: recipient.clone(),
            amount,
            signatures: vec![],
        };
        
        // Emit event for validators to sign
        emit_event(BridgeEvent::TransferInitiated(tx.clone()));
        
        Ok(tx)
    }
    
    pub fn complete_transfer(&self, tx: BridgeTransaction) -> Result<()> {
        // Verify validator signatures
        ensure!(
            tx.signatures.len() >= self.threshold as usize,
            Error::InsufficientSignatures
        );
        
        for sig in &tx.signatures {
            let validator = recover_signer(&tx.message(), sig)?;
            ensure!(self.validators.contains(&validator), Error::InvalidValidator);
        }
        
        // Mint tokens on destination chain
        mint_tokens(&tx.recipient, tx.amount)?;
        
        Ok(())
    }
}
```

## Production Architecture

### Complete System Design

```
                    ┌─────────────────────────────────┐
                    │      Load Balancer (HAProxy)    │
                    └──────────────┬──────────────────┘
                                   │
              ┌────────────────────┼────────────────────┐
              │                    │                    │
    ┌─────────▼────────┐  ┌───────▼────────┐  ┌───────▼────────┐
    │   RPC Node 1     │  │   RPC Node 2   │  │   RPC Node 3   │
    │  (Read-heavy)    │  │  (Read-heavy)  │  │  (Read-heavy)  │
    └─────────┬────────┘  └────────┬───────┘  └────────┬───────┘
              │                    │                    │
              └────────────────────┼────────────────────┘
                                   │
                    ┌──────────────▼──────────────┐
                    │      Validator Nodes        │
                    │   (Consensus Participants)  │
                    └──────────────┬──────────────┘
                                   │
              ┌────────────────────┼────────────────────┐
              │                    │                    │
    ┌─────────▼────────┐  ┌───────▼────────┐  ┌───────▼────────┐
    │  Archive Node 1  │  │ Archive Node 2 │  │ Archive Node 3 │
    │ (Full history)   │  │ (Full history) │  │ (Full history) │
    └──────────────────┘  └────────────────┘  └────────────────┘
                                   │
                    ┌──────────────▼──────────────┐
                    │      Database Cluster       │
                    │   (PostgreSQL + TimescaleDB)│
                    └─────────────────────────────┘
```

### Configuration Management

```rust
// config.toml
#[derive(Deserialize)]
pub struct BlockchainConfig {
    pub network: NetworkConfig,
    pub consensus: ConsensusConfig,
    pub storage: StorageConfig,
    pub monitoring: MonitoringConfig,
}

#[derive(Deserialize)]
pub struct NetworkConfig {
    pub listen_addr: SocketAddr,
    pub external_addr: Option<SocketAddr>,
    pub bootstrap_peers: Vec<String>,
    pub max_peers: usize,
}

#[derive(Deserialize)]
pub struct ConsensusConfig {
    pub algorithm: String,  // "babe", "aura", "pow"
    pub block_time: u64,
    pub finality_lag: u32,
}

impl BlockchainConfig {
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content).map_err(Into::into)
    }
}
```

### High Availability Setup

```rust
pub struct HACoordinator {
    primary: NodeId,
    replicas: Vec<NodeId>,
    health_checker: Arc<HealthChecker>,
}

impl HACoordinator {
    pub async fn run(&mut self) {
        loop {
            // Check primary health
            if !self.health_checker.is_healthy(&self.primary).await {
                // Promote replica to primary
                self.failover().await;
            }
            
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    }
    
    async fn failover(&mut self) {
        let new_primary = self.select_best_replica().await;
        
        log::warn!("Failing over from {:?} to {:?}", self.primary, new_primary);
        
        // Update load balancer
        self.update_load_balancer(new_primary).await;
        
        // Update local state
        self.replicas.push(self.primary);
        self.primary = new_primary;
        self.replicas.retain(|id| *id != new_primary);
    }
}
```

This architecture provides a solid foundation for building production-grade blockchain systems with Rust.
