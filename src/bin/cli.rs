use log::{debug, error};
use paymo::{cli, client, config, init_logger, Error};

fn main() -> paymo::Result<()> {
    init_logger();

    debug!("PID: {}", std::process::id());

    let opts = cli::Opts::try_init();

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

    debug!("{conf:#?}");

    client::Client::from_opts(opts).add_conf(conf)?.run()?;

    Ok(())
}
