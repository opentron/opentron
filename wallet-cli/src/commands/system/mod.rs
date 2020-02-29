//! Commands for system contracts.

use clap::ArgMatches;

use crate::error::Error;

mod freeze;
mod proposal;
mod witness;

pub fn main(matches: &ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("vote_witness", Some(arg_matches)) => witness::vote(arg_matches),
        ("create_witness", Some(arg_matches)) => witness::create(arg_matches),
        ("update_witness", Some(arg_matches)) => witness::update(arg_matches),
        ("withdraw_reward", Some(arg_matches)) => witness::withdraw_reward(arg_matches),
        ("update_brokerage", Some(arg_matches)) => witness::update_brokerage(arg_matches),
        ("create_proposal", Some(arg_matches)) => proposal::create(arg_matches),
        ("approve_proposal", Some(arg_matches)) => proposal::approve(true, arg_matches),
        ("disapprove_proposal", Some(arg_matches)) => proposal::approve(false, arg_matches),
        ("delete_proposal", Some(arg_matches)) => proposal::delete(arg_matches),
        ("freeze", Some(arg_matches)) => freeze::freeze(arg_matches),
        ("unfreeze", Some(arg_matches)) => freeze::unfreeze(arg_matches),

        _ => {
            eprintln!("{}", matches.usage());
            Err(Error::Runtime("error parsing command line"))
        }
    }
}
