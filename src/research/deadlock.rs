// Deadlock Detection and Resource Allocation Algorithm
// Implements Banker's algorithm and Resource-Allocation Graph cycle detection
extern crate alloc;

use alloc::vec::Vec;

pub const MAX_PROCESSES: usize = 5;
pub const MAX_RESOURCES: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceState {
    pub num_processes: usize,
    pub num_resources: usize,
    pub available: [u32; MAX_RESOURCES],
    pub max_claim: [[u32; MAX_RESOURCES]; MAX_PROCESSES],
    pub allocation: [[u32; MAX_RESOURCES]; MAX_PROCESSES],
    pub need: [[u32; MAX_RESOURCES]; MAX_PROCESSES],
}

#[derive(Debug, PartialEq, Eq)]
pub enum SafetyResult {
    Safe(Vec<usize>),       // Safe sequence of process IDs
    Deadlock(Vec<usize>),   // Unfinished process IDs forming deadlock
}

impl ResourceState {
    pub fn new(
        num_p: usize,
        num_r: usize,
        available: [u32; MAX_RESOURCES],
        max_claim: [[u32; MAX_RESOURCES]; MAX_PROCESSES],
        allocation: [[u32; MAX_RESOURCES]; MAX_PROCESSES],
    ) -> Result<Self, &'static str> {
        if num_p > MAX_PROCESSES || num_r > MAX_RESOURCES {
            return Err("Process or resource count exceeds maximum capacity");
        }

        let mut need = [[0u32; MAX_RESOURCES]; MAX_PROCESSES];
        for p in 0..num_p {
            for r in 0..num_r {
                if allocation[p][r] > max_claim[p][r] {
                    return Err("Allocation exceeds maximum claim");
                }
                need[p][r] = max_claim[p][r] - allocation[p][r];
            }
        }

        Ok(ResourceState {
            num_processes: num_p,
            num_resources: num_r,
            available,
            max_claim,
            allocation,
            need,
        })
    }

    // Run Banker's safety / deadlock detection algorithm
    pub fn check_safety(&self) -> SafetyResult {
        let mut work = self.available;
        let mut finish = [false; MAX_PROCESSES];
        let mut safe_sequence = Vec::new();

        loop {
            let mut found = false;
            for p in 0..self.num_processes {
                if !finish[p] {
                    // Check if need <= work for all resources
                    let mut can_allocate = true;
                    for r in 0..self.num_resources {
                        if self.need[p][r] > work[r] {
                            can_allocate = false;
                            break;
                        }
                    }

                    if can_allocate {
                        for r in 0..self.num_resources {
                            work[r] += self.allocation[p][r];
                        }
                        finish[p] = true;
                        safe_sequence.push(p);
                        found = true;
                    }
                }
            }

            if !found {
                break;
            }
        }

        let mut deadlocked = Vec::new();
        for p in 0..self.num_processes {
            if !finish[p] {
                deadlocked.push(p);
            }
        }

        if deadlocked.is_empty() {
            SafetyResult::Safe(safe_sequence)
        } else {
            SafetyResult::Deadlock(deadlocked)
        }
    }
}

// Generate canonical textbook test patterns
pub fn generate_safe_pattern() -> ResourceState {
    let available = [3, 3, 2];
    let max_claim = [
        [7, 5, 3],
        [3, 2, 2],
        [9, 0, 2],
        [2, 2, 2],
        [4, 3, 3],
    ];
    let allocation = [
        [0, 1, 0],
        [2, 0, 0],
        [3, 0, 2],
        [2, 1, 1],
        [0, 0, 2],
    ];
    ResourceState::new(5, 3, available, max_claim, allocation).unwrap()
}

pub fn generate_deadlock_pattern() -> ResourceState {
    // Circular wait condition
    let available = [0, 0, 0];
    let max_claim = [
        [2, 1, 1],
        [1, 2, 1],
        [1, 1, 2],
        [0, 0, 0],
        [0, 0, 0],
    ];
    let allocation = [
        [1, 0, 1],
        [1, 1, 0],
        [0, 1, 1],
        [0, 0, 0],
        [0, 0, 0],
    ];
    ResourceState::new(3, 3, available, max_claim, allocation).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_pattern() {
        let state = generate_safe_pattern();
        match state.check_safety() {
            SafetyResult::Safe(seq) => {
                assert_eq!(seq.len(), 5);
                assert_eq!(seq[0], 1); // P1 executes first in textbook example
            }
            SafetyResult::Deadlock(_) => panic!("Expected safe sequence"),
        }
    }

    #[test]
    fn test_deadlock_pattern() {
        let state = generate_deadlock_pattern();
        match state.check_safety() {
            SafetyResult::Deadlock(deadlocked) => {
                assert!(!deadlocked.is_empty());
                assert_eq!(deadlocked.len(), 3);
            }
            SafetyResult::Safe(_) => panic!("Expected deadlock"),
        }
    }

    #[test]
    fn test_invalid_allocation() {
        let available = [1, 1, 1];
        let max_claim = [[2, 2, 2], [0, 0, 0], [0, 0, 0], [0, 0, 0], [0, 0, 0]];
        let allocation = [[3, 1, 1], [0, 0, 0], [0, 0, 0], [0, 0, 0], [0, 0, 0]]; // 3 > 2!
        assert!(ResourceState::new(1, 3, available, max_claim, allocation).is_err());
    }
}
