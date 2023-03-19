#[derive(clap::ValueEnum, Clone, Debug)]
pub enum Role {
    #[value(alias = "Sender", rename_all = "PascalCase")]
    Alice,

    #[value(alias = "Receiver", rename_all = "PascalCase")]
    Bob,
}
