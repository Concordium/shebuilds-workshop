//! A voting smart contract example.
//!
//! # Description
//! A contract that allows for conducting an election with several voting
//! options. An `end_time` is set when the election is initialized. Only
//! accounts are eligible to vote. Each account can change its
//! selected voting option as often as it desires until the `end_time` is
//! reached. No voting will be possible after the `end_time`.
//!
//! # Operations
//! The contract allows for
//!  - `initializing` the election;
//!  - `vote` for one of the voting options;
//!  - `view` general information about the election and the tally.
//!
//! # Tests
//! The tests exist in the `./tests/tests.rs` file.
//!
//! Note: Vec<VotingOption> (among other variables) is an input parameter to the
//! `init` function. Since there is a limit to the parameter size (65535 Bytes),
//! the size of the Vec<VotingOption> is limited.
//! https://developer.concordium.software/en/mainnet/smart-contracts/general/contract-instances.html#limits

use concordium_std::{collections::BTreeMap, *};

/// The human-readable description of a voting option.
pub type VotingOption = String;
/// The voting options are stored in a vector. The vector index is used to refer
/// to a specific voting option.
pub type VoteIndex = u32;
/// Number of votes.
pub type VoteCount = u32;

/// The parameter type for the contract function `init`.
/// Takes a description, the voting options, and the `end_time` to start the
/// election.
#[derive(Serialize, SchemaType)]
pub struct InitParameter {
    /// The description of the election.
    pub description: String,
    /// A vector of all voting options.
    pub options: Vec<VotingOption>,
    /// The last timestamp that an account can vote.
    /// The election is open from the point in time that this smart contract is
    /// initialized until the `end_time`.
    pub end_time: Timestamp,
}

/// The `return_value` type of the contract function `view`.
/// Returns a description, the `end_time`, the voting options as a vector, and
/// the number of voting options of the current election.
/// Also returns the tally of votes.
#[derive(Serialize, SchemaType)]
pub struct VotingView {
    /// The description of the election.
    pub description: String,
    /// The last timestamp that an account can vote.
    /// The election is open from the point in time that this smart contract is
    /// initialized until the `end_time`.
    pub end_time: Timestamp,
    /// The map connects the index of a voting option to the number of votes
    /// it received so far.
    pub tally: BTreeMap<VotingOption, VoteCount>,
}

/// The contract state
#[derive(Serialize, Clone)]
struct State {
    /// The description of the election.
    description: String,
    /// The map connects a voter to the index of the voted-for voting option.
    ballots: BTreeMap<AccountAddress, VoteIndex>,
    /// The last timestamp that an account can vote.
    /// The election is open from the point in time that this smart contract is
    /// initialized until the `end_time`.
    end_time: Timestamp,
    /// A vector of all voting options.
    options: Vec<VotingOption>,
}

/// The different errors that the `vote` function can produce.
#[derive(Reject, Serialize, PartialEq, Eq, Debug, SchemaType)]
pub enum VotingError {
    /// Raised when parsing the parameter failed.
    #[from(ParseError)]
    ParsingFailed,
    /// Raised when the vote is placed after the election has ended.
    VotingFinished,
    /// Raised when voting for a voting option that does not exist.
    InvalidVotingOption,
    /// Raised when a smart contract tries to participate in the election. Only
    /// accounts are allowed to vote.
    ContractVoter,
}

/// A custom alias type for the `Result` type with the error type fixed to
/// `VotingError`.
pub type VotingResult<T> = Result<T, VotingError>;

// Contract functions

/// Initialize the contract instance and start the election.
/// A description, the vector of all voting options, and an `end_time`
/// have to be provided.
#[init(contract = "voting", parameter = "InitParameter")]
fn init(ctx: &InitContext, _state_builder: &mut StateBuilder) -> InitResult<State> {
    // Parse the parameter.
    let param: InitParameter = ctx.parameter_cursor().get()?;

    // Set the state.
    Ok(State {
        description: param.description,
        ballots: BTreeMap::new(),
        end_time: param.end_time,
        options: param.options,
    })
}

/// Enables accounts to vote for a specific voting option. Each account can
/// change its selected voting option with this function as often as it desires
/// until the `end_time` is reached.
///
/// It rejects if:
/// - It fails to parse the parameter.
/// - A contract tries to vote.
/// - It is past the `end_time`.
#[receive(
    contract = "voting",
    name = "vote",
    mutable,
    parameter = "VotingOption",
    error = "VotingError"
)]
fn vote(ctx: &ReceiveContext, host: &mut Host<State>) -> VotingResult<()> {
    // Check that the election hasn't finished yet.
    if ctx.metadata().slot_time() > host.state().end_time {
        return Err(VotingError::VotingFinished);
    }

    // Ensure that the sender is an account.
    let acc = match ctx.sender() {
        Address::Account(acc) => acc,
        Address::Contract(_) => return Err(VotingError::ContractVoter),
    };

    // Parse the parameter.
    let new_vote: VotingOption = ctx.parameter_cursor().get()?;
    // Find the vote index in state.options. Or return an error, if it doesn't exist.
    let new_vote_index = match host.state().options.iter().position(|o| *o == new_vote) {
        Some(vote_index) => vote_index as u32,
        _ => return Err(VotingError::InvalidVotingOption),
    };

    // Insert or replace the vote for the account.
    host.state_mut()
        .ballots
        .entry(acc)
        .and_modify(|old_vote_index| *old_vote_index = new_vote_index)
        .or_insert(new_vote_index);

    Ok(())
}

/// Get the election information.
#[receive(contract = "voting", name = "view", return_value = "VotingView")]
fn view(_ctx: &ReceiveContext, host: &Host<State>) -> ReceiveResult<VotingView> {
    // Get information from the state.
    let description = host.state().description.clone();
    let end_time = host.state().end_time;
    let options = host.state().options.clone();
    let mut tally = BTreeMap::new();

    // Sum up the ballots to a tally.
    // Looping over data that can be changed by users should be avoided in
    // production, as there might be so many ballots that the loop cannot be
    // processed in time.
    for (_, vote_index) in host.state().ballots.iter() {
        // Get the VotingOption (String).
        let voting_option = options[*vote_index as usize].clone();
        // Increment the existing value or insert 1.
        tally
            .entry(voting_option)
            .and_modify(|current_count| *current_count += 1)
            .or_insert(1);
    }

    // Return the election information.
    Ok(VotingView {
        description,
        end_time,
        tally,
    })
}
