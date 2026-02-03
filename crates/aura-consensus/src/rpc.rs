use serde::{Serialize, Deserialize};

/// Sent by Candidates to gather votes
#[derive(Debug, Serialize, Deserialize)]
pub struct RequestVote {
    pub term: u64,
    pub candidate_id: u32,
    pub last_log_index: u64, // Used later for safety
    pub last_log_term: u64,
}

/// The Response from other nodes
#[derive(Debug, Serialize, Deserialize)]
pub struct RequestVoteResponse {
    pub term: u64,
    pub vote_granted: bool,
}

/// Sent by Leaders to say "I am alive" (and replicate logs)
#[derive(Debug, Serialize, Deserialize)]
pub struct AppendEntries {
    pub term: u64,
    pub leader_id: u32,
    // Log entries will go here later
}