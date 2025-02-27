use clap::{Parser, Subcommand};
use shadow_rs::shadow;

shadow!(build);

use code_maven::drafts::list_drafts;
use code_maven::new::new_site;
use code_maven::notifications::cm_sendgrid;
use code_maven::recent::get_recent;
use code_maven::todo::list_todo;
use code_maven::web::web;

#[derive(Parser, Debug)]
#[command(version = build::CLAP_LONG_VERSION)]
struct Cli {
    #[arg(long, default_value = "warn")]
    log: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Web {
        #[arg(long, default_value = ".")]
        root: String,

        #[arg(long, default_value = "")]
        config: String,

        #[arg(long, default_value = "")]
        pages: String,

        #[arg(long, default_value = "_site")]
        outdir: String,
    },

    Recent {
        #[arg(long, default_value = ".")]
        root: String,

        #[arg(long, default_value = "")]
        pages: String,

        #[arg(long, default_value = "")]
        days: String,
    },

    Sendgrid {
        #[arg(long, default_value = ".")]
        root: String,

        #[arg(long)]
        tofile: String,

        #[arg(long)]
        mail: String,
    },

    Todo {
        #[arg(long, default_value = ".")]
        root: String,

        #[arg(long, default_value = "")]
        pages: String,
    },

    New {
        #[arg(long)]
        root: String,
    },

    Drafts {
        #[arg(long, default_value = ".")]
        root: String,

        #[arg(long, default_value = "")]
        pages: String,
    },
}

fn main() {
    let args = Cli::parse();
    let log_level = match args.log.as_str() {
        //"debug" => log::Level::Debug,
        "info" => log::Level::Info,
        "warn" => log::Level::Warn,
        _ => log::Level::Debug,
    };

    simple_logger::init_with_level(log_level).unwrap();

    let result = match &args.command {
        Commands::Web {
            root,
            config,
            pages,
            outdir,
        } => web(root, config, pages, outdir),
        Commands::Recent { root, pages, days } => get_recent(root, pages, days),
        Commands::New { root } => new_site(root),
        Commands::Sendgrid { root, tofile, mail } => cm_sendgrid(root, mail, tofile),
        Commands::Todo { root, pages } => list_todo(root, pages),
        Commands::Drafts { root, pages } => list_drafts(root, pages),
    };

    match result {
        Ok(()) => {}
        Err(error) => {
            log::error!("{}", error);
            std::process::exit(1);
        }
    };
}
