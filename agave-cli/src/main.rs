mod util;

use std::io::{self, Write};

use agave::{
    apply_replaces, apply_replaces_lines, create_jenga_op_for_cluster, get_descendants_by_tag,
    get_svd_parent_chain, roxmltree::Document, CLUSTER_TAG,
};
use clap::{Parser, Subcommand, ValueEnum};
use itertools::Itertools;
use util::{read_file, write_file};

#[derive(Parser)]
#[command(version, about, long_about = None, author = clap::crate_authors!(), subcommand_required = true)]
struct Cli {
    #[arg(long)]
    svd: String,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Clone)]
enum TokenStrategy {
    Bytes,
    Lines,
}

#[derive(Subcommand)]
enum Command {
    Jenga {
        /// A file containing lines representing clusters to be jenga'd
        #[arg(short = 'f', long)]
        lines_file: Option<String>,

        #[arg(short, long = "output")]
        ofile: Option<String>,

        /// Produce output line-by-line or byte-by-byte
        ///
        /// Line-by-line produces prettier output at the moment but it can fail if the input format
        /// is unexpected, so check the output!
        #[arg(short, long = "strategy")]
        token_strategy: TokenStrategy,
    },
}

impl ValueEnum for TokenStrategy {
    fn value_variants<'a>() -> &'a [Self] {
        &[TokenStrategy::Bytes, TokenStrategy::Lines]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        use clap::builder::PossibleValue;
        match self {
            TokenStrategy::Bytes => Some(PossibleValue::new("bytes")),
            TokenStrategy::Lines => Some(PossibleValue::new("lines")),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if let Some(cmd) = cli.command {
        match cmd {
            Command::Jenga {
                lines_file,
                ofile,
                token_strategy,
            } => {
                let xml = read_file(cli.svd);
                let doc = Document::parse(&xml)?;

                let select_strs = if let Some(lines_file) = lines_file {
                    parse_lines(read_file(lines_file))
                } else {
                    panic!("must supply lines_file (for now)");
                };

                let selected_clusters = get_descendants_by_tag(doc.root(), CLUSTER_TAG)
                    .iter()
                    .filter(|node| select_strs.contains(&get_svd_parent_chain(**node).join(".")))
                    .cloned()
                    .collect_vec();

                let replaces = selected_clusters
                    .into_iter()
                    .map(create_jenga_op_for_cluster)
                    .collect_vec();

                let output = match token_strategy {
                    TokenStrategy::Bytes => apply_replaces(&doc, &replaces),
                    TokenStrategy::Lines => apply_replaces_lines(&doc, &replaces),
                };

                if let Some(ofile) = ofile {
                    write_file(ofile, &output);
                } else {
                    io::stdout().write_all(&output).unwrap();
                }
            }
        }
    }

    Ok(())
}

fn parse_lines(lines_file: String) -> Vec<String> {
    lines_file
        .lines()
        // Remove anything and everything after first comment character
        .map(|line| line.split('#').nth(0).unwrap())
        // Strip whitespace
        .map(|line| line.trim())
        // Remove empty
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect_vec()
}
