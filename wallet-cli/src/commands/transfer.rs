use clap::ArgMatches;
use keys::Address;
use proto::core::TransferContract;

use crate::error::Error;
use crate::utils::trx;
use crate::utils::trx::TransactionHandler;

pub fn main<'a>(matches: &'a ArgMatches<'a>) -> Result<(), Error> {
    let sender = matches
        .value_of("SENDER")
        .and_then(|s| s.parse::<Address>().ok())
        .ok_or(Error::Runtime("wrong sender address format"))?;
    let recipient = matches
        .value_of("RECIPIENT")
        .and_then(|s| s.parse::<Address>().ok())
        .ok_or(Error::Runtime("wrong recipient address format"))?;
    let amount = matches.value_of("AMOUNT").expect("required in cli.yml; qed");
    let memo = matches.value_of("MEMO").unwrap_or("").as_bytes().to_owned();

    let transfer_contract = TransferContract {
        owner_address: sender.to_bytes().to_owned(),
        to_address: recipient.to_bytes().to_owned(),
        amount: trx::parse_amount_with_surfix(amount, "TRX", 6)?,
        ..Default::default()
    };

    eprintln!("sender:    {:}", sender);
    eprintln!("recipient: {:}", recipient);

    TransactionHandler::handle(transfer_contract, matches)
        .map_raw_transaction(move |raw| raw.set_data(memo.clone()))
        .run()
}
