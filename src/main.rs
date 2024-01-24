use clap::{Parser, Subcommand};

use code_maven::drafts::list_drafts;
use code_maven::notifications::cm_sendgrid;
use code_maven::todo::list_todo;
use code_maven::web::web;

#[derive(Parser, Debug)]
#[command(version)]
struct Cli {
    #[arg(long)]
    debug: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Web {
        #[arg(long, default_value = ".")]
        root: String,

        #[arg(long, default_value = "")]
        pages: String,

        #[arg(long, default_value = "_site")]
        outdir: String,

        #[arg(long, default_value = "")]
        email: String,
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

    Drafts {
        #[arg(long, default_value = ".")]
        root: String,

        #[arg(long, default_value = "")]
        pages: String,
    },
}

fn main() {
    let args = Cli::parse();
    let log_level = if args.debug {
        log::Level::Debug
    } else {
        log::Level::Warn
    };
    simple_logger::init_with_level(log_level).unwrap();

    match &args.command {
        Commands::Web {
            root,
            pages,
            outdir,
            email,
        } => web(root, pages, outdir, email),
        Commands::Sendgrid { root, tofile, mail } => cm_sendgrid(root, mail, tofile),
        Commands::Todo { root, pages } => list_todo(root, pages),
        Commands::Drafts { root, pages } => list_drafts(root, pages),
    }
}
