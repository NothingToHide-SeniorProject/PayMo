use log::{debug, error};
use paymo::{init_logger, peerd};

fn main() -> paymo::Result<()> {
    init_logger();

    debug!("PID: {}", std::process::id());

    let opts = peerd::Opts::try_init();

    if let Err(err) = opts {
        error!("{err}");
        return Err(err);
    }

    peerd::Peerd::new().run(opts.unwrap())?;

    Ok(())
}
