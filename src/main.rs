use clap::{Parser, Subcommand};

use code_maven::cm_sendgrid::cm_sendgrid;
use code_maven::todo::list_todo;
use code_maven::web::web;

#[derive(Parser, Debug)]
#[command(version)]
struct Cli {
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
    },
}

fn main() {
    let args = Cli::parse();
    //println!("{:?}", &args);
    simple_logger::init_with_env().unwrap();
    log::info!("Generate pages");

    match &args.command {
        Commands::Web {
            root,
            pages,
            outdir,
            email,
        } => web(root, pages, outdir, email),
        Commands::Sendgrid { root, tofile, mail } => cm_sendgrid(root, mail, tofile),
        Commands::Todo { root } => list_todo(root),
    }
}
