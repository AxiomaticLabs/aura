pub mod rpc;
pub mod state;

use std::time::Duration;
use rand::Rng;
use tracing::{info, warn};

/// The Heartbeat interval (Leader pings followers every 150ms)
/// const HEARTBEAT_INTERVAL: u64 = 150; 

/// Min/Max Election Timeout (Randomized 300ms - 600ms)
const ELECTION_TIMEOUT_MIN: u64 = 300;
const ELECTION_TIMEOUT_MAX: u64 = 600;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Role {
    Follower,
    Candidate,
    Leader,
}

pub struct RaftNode {
    pub id: u32,
    pub current_term: u64,
    pub voted_for: Option<u32>,
    pub role: Role,
    
    // Timer state
    last_heartbeat: std::time::Instant,
    election_timeout: std::time::Duration,
}

impl RaftNode {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            current_term: 0,
            voted_for: None,
            role: Role::Follower, // Everyone starts as a Follower
            last_heartbeat: std::time::Instant::now(),
            election_timeout: Self::random_timeout(),
        }
    }

    /// The Main Loop Tick: Checks if we need to start an election
    pub fn tick(&mut self) {
        if self.role == Role::Leader {
            // Leaders don't have election timeouts
            return;
        }

        // Check if Election Timeout expired
        if self.last_heartbeat.elapsed() > self.election_timeout {
            self.start_election();
        }
    }

    /// Transition: Follower -> Candidate
    fn start_election(&mut self) {
        info!("Node {}: Election Timeout! Becoming CANDIDATE.", self.id);
        
        self.role = Role::Candidate;
        self.current_term += 1;          // Increment Term
        self.voted_for = Some(self.id);  // Vote for self
        self.last_heartbeat = std::time::Instant::now(); // Reset timer
        self.election_timeout = Self::random_timeout();  // Pick new random timeout

        // TODO: Send RequestVote RPC to all other peers
        self.request_votes();
    }

    fn request_votes(&self) {
        // This is where we will broadcast packets in the next step
        warn!("Node {}: Term {} - Requesting Votes...", self.id, self.current_term);
    }

    /// Reset the timer (Called when we get a valid heartbeat from Leader)
    pub fn reset_election_timer(&mut self) {
        self.last_heartbeat = std::time::Instant::now();
        self.election_timeout = Self::random_timeout();
    }

    fn random_timeout() -> Duration {
        let mut rng = rand::thread_rng();
        let millis = rng.gen_range(ELECTION_TIMEOUT_MIN..ELECTION_TIMEOUT_MAX);
        Duration::from_millis(millis)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_election_timeout_trigger() {
        // 1. Create a Follower Node
        let mut node = RaftNode::new(1);
        assert_eq!(node.role, Role::Follower);
        assert_eq!(node.current_term, 0);

        // 2. Simulate waiting (Sleep longer than max timeout)
        // Max timeout is 600ms, so we sleep 650ms
        thread::sleep(Duration::from_millis(650));

        // 3. Tick the logic
        node.tick();

        // 4. Verify it promoted itself to Candidate
        assert_eq!(node.role, Role::Candidate);
        assert_eq!(node.current_term, 1); // Term increased
        assert_eq!(node.voted_for, Some(1)); // Voted for self
        
        println!("✅ Node 1 successfully started election due to timeout.");
    }
}
