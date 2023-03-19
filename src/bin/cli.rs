use log::{debug, error};
use paymo::{cli, config, init_logger, Error};

fn main() -> paymo::Result<()> {
    init_logger();

    debug!("PID: {}", std::process::id());

    let opts = cli::Opts::try_init();

    // TODO parse Config
    // TODO create ChannelConfig from Opts and Config (but maybe inside start_protocol)
    // TODO start_protocol should also create link (from ChannelConfig)?
    // TODO call start_protocol

    match opts {
        Err(Error::Cli(cli::Error::Cmd(err))) => clap::Error::from(err).exit(),
        Err(err) => {
            error!("{err}");
            return Err(err);
        }
        _ => (),
    };

    let opts = opts.unwrap();

    let conf = config::Config::from_path(&opts.config_file);
    if let Err(err) = conf {
        error!("{err}");
        return Err(err);
    }

    let conf = conf.unwrap();

    Ok(())
}
