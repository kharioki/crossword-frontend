use near_sdk::collections::{LookupMap, UnorderedSet};
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    log,
    serde::{Deserialize, Serialize},
    AccountId, PanicOnDefault, Promise,
};
use near_sdk::{env, near_bindgen};

// 5 Ⓝ in yoctoNEAR
const PRIZE_AMOUNT: u128 = 5_000_000_000_000_000_000_000_000;

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum AnswerDirection {
    Across,
    Down,
}

/// The origin (0,0) starts at the top left side of the square
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct CoordinatePair {
    x: u8,
    y: u8,
}

// {"num": 1, "start": {"x": 19, "y": 31}, "direction": "Across", "length": 8, "clue": "not far but"}
// We'll have the clue stored on-chain for now for simplicity.
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Answer {
    num: u8,
    start: CoordinatePair,
    direction: AnswerDirection,
    length: u8,
    clue: String,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum PuzzleStatus {
    Unsolved,
    Solved { memo: String },
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct UnsolvedPuzzles {
    puzzles: Vec<JsonPuzzle>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonPuzzle {
    /// The human-readable (not in bytes) hash of the solution
    solution_hash: String,
    status: PuzzleStatus,
    answer: Vec<Answer>,
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct Puzzle {
    status: PuzzleStatus,
    /// Use the CoordinatePair assuming the origin is (0, 0) in the top left side of the puzzle.
    answer: Vec<Answer>,
}

/// Regarding PanicOnDefault:
/// When you want to have a "new" function initialize a smart contract,
/// you'll likely want to follow this pattern of having a default implementation that panics,
/// directing the user to call the initialization method. (The one with the #[init] macro)
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Crossword {
    owner_id: AccountId,
    puzzles: LookupMap<String, Puzzle>,
    unsolved_puzzles: UnorderedSet<String>,
}

#[near_bindgen]
impl Crossword {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        Self {
            owner_id,
            puzzles: LookupMap::new(b"c"),
            unsolved_puzzles: UnorderedSet::new(b"u"),
        }
    }

    pub fn submit_solution(&mut self, solution: String, memo: String) {
        let hashed_input = env::sha256(solution.as_bytes());
        let hashed_input_hex = hex::encode(&hashed_input);

        // Check to see if the hashed answer is among the puzzles
        let mut puzzle = self
            .puzzles
            .get(&hashed_input_hex)
            .expect("ERR_NOT_CORRECT_ANSWER");

        // Check if the puzzle is already solved. If it's unsolved, set the status to solved,
        //   then proceed to update the puzzle and pay the winner.
        puzzle.status = match puzzle.status {
            PuzzleStatus::Unsolved => PuzzleStatus::Solved { memo: memo.clone() },
            _ => {
                env::panic_str("ERR_PUZZLE_SOLVED");
            }
        };

        // Reinsert the puzzle back in after we modified the status:
        self.puzzles.insert(&hashed_input_hex, &puzzle);
        // Remove from the list of unsolved ones
        self.unsolved_puzzles.remove(&hashed_input_hex);

        log!(
            "Puzzle with solution hash {} solved, with memo: {}",
            hashed_input_hex,
            memo
        );

        // Transfer the prize money to the winner
        Promise::new(env::predecessor_account_id()).transfer(PRIZE_AMOUNT);
    }

    /// Get the hash of a crossword puzzle solution from the unsolved_puzzles
    pub fn get_solution(&self, puzzle_index: u32) -> Option<String> {
        let mut index = 0;
        for puzzle_hash in self.unsolved_puzzles.iter() {
            if puzzle_index == index {
                return Some(puzzle_hash);
            }
            index += 1;
        }
        // Did not find that index
        None
    }

    // adding a new crossword puzzle
    pub fn new_puzzle(&mut self, solution_hash: String, answers: Vec<Answer>) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner_id,
            "Only owner can create a new puzzle"
        );
        let existing = self.puzzles.insert(
            &solution_hash,
            &Puzzle {
                status: PuzzleStatus::Unsolved,
                answer: answers,
            },
        );

        assert!(existing.is_none(), "Puzzle with that key already exists");
        self.unsolved_puzzles.insert(&solution_hash);
    }
}

/*
 * the rest of this file sets up unit tests
 * to run these, the command will be:
 * cargo test --package rust-template -- --nocapture
 * Note: 'rust-template' comes from Cargo.toml's 'name' key
 */

// use the attribute below for unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{get_logs, VMContextBuilder};
    use near_sdk::{testing_env, AccountId};

    // part of writing unit tests is setting up a mock context
    // provide a `predecessor` here, it'll modify the default context
    fn get_context(predecessor: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor);
        builder
    }

    #[test]
    fn debug_get_hash() {
        // Basic set up for a unit test
        testing_env!(VMContextBuilder::new().build());

        // Using a unit test to rapidly debug and iterate
        let debug_solution = "near nomicon ref finance";
        let debug_hash_bytes = env::sha256(debug_solution.as_bytes());
        let debug_hash_string = hex::encode(debug_hash_bytes);
        println!("Let's debug: {:?}", debug_hash_string);
    }

    #[test]
    fn check_guess_solution() {
        // Get Alice as an account ID
        let alice = AccountId::new_unchecked("alice.testnet".to_string());
        // Set up the testing context and unit test environment
        let context = get_context(alice);
        testing_env!(context.build());

        // Set up contract object and call the new method
        let mut contract = Contract::new(
            "69c2feb084439956193f4c21936025f14a5a5a78979d67ae34762e18a7206a0f".to_string(),
        );
        let mut guess_result = contract.guess_solution("wrong answer here".to_string());
        assert!(!guess_result, "Expected a failure from the wrong guess");
        assert_eq!(get_logs(), ["Try again."], "Expected a failure log.");
        guess_result = contract.guess_solution("near nomicon ref finance".to_string());
        assert!(guess_result, "Expected the correct answer to return true.");
        assert_eq!(
            get_logs(),
            ["Try again.", "You guessed right!"],
            "Expected a successful log after the previous failed log."
        );
    }
}
