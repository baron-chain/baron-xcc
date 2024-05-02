use clap::Parser;
use std::path::PathBuf;

use crate::{chain_spec, eth::EthConfiguration};

/// Sub-commands supported by the collator.
#[derive(Debug, Parser)]
pub enum Subcommand {
    /// Export the genesis state of the parachain.
    #[clap(name = "export-genesis-state")]
    ExportGenesisState(cumulus_client_cli::ExportGenesisStateCommand),

    /// Export the genesis wasm of the parachain.
    #[clap(name = "export-genesis-wasm")]
    ExportGenesisWasm(cumulus_client_cli::ExportGenesisWasmCommand),

    /// Export the metadata.
    #[clap(name = "export-metadata")]
    ExportMetadata(ExportMetadataCommand),

    /// Build a chain specification.
    BuildSpec(sc_cli::BuildSpecCmd),

    /// Validate blocks.
    CheckBlock(sc_cli::CheckBlockCmd),

    /// Export blocks.
    ExportBlocks(sc_cli::ExportBlocksCmd),

    /// Export the state of a given block into a chain spec.
    ExportState(sc_cli::ExportStateCmd),

    /// Import blocks.
    ImportBlocks(sc_cli::ImportBlocksCmd),

    /// Remove the whole chain.
    PurgeChain(cumulus_client_cli::PurgeChainCmd),

    /// Revert the chain to a previous state.
    Revert(sc_cli::RevertCmd),

    /// The custom benchmark subcommmand benchmarking runtime pallets.
    #[clap(subcommand)]
    Benchmark(frame_benchmarking_cli::BenchmarkCmd),

    /// Try some command against runtime state.
    #[cfg(feature = "try-runtime")]
    TryRuntime(try_runtime_cli::TryRuntimeCmd),

    /// Try some command against runtime state. Note: `try-runtime` feature must be enabled.
    #[cfg(not(feature = "try-runtime"))]
    TryRuntime,
}

/// Command for exporting the metadata.
#[derive(Debug, Parser)]
pub struct ExportMetadataCommand {
    /// Output file name or stdout if unspecified.
    #[clap(action)]
    pub output: Option<PathBuf>,

    /// Write output in binary. Default is to write in hex.
    #[clap(short, long)]
    pub raw: bool,

    /// The name of the runtime to retrieve the metadata from.
    #[clap(long)]
    pub runtime: RuntimeName,
}

#[derive(clap::ValueEnum, Debug, Clone)]
pub enum RuntimeName {
    Interlay,
    Kintsugi,
}

#[derive(Debug, Parser)]
#[clap(
    propagate_version = true,
    args_conflicts_with_subcommands = true,
    subcommand_negates_reqs = true
)]
pub struct Cli {
    #[clap(subcommand)]
    pub subcommand: Option<Subcommand>,

    #[clap(flatten)]
    pub run: cumulus_client_cli::RunCmd,

    #[clap(flatten)]
    pub eth: EthConfiguration,

    /// Relaychain arguments
    #[clap(raw = true)]
    pub relaychain_args: Vec<String>,

    /// Instant block sealing
    ///
    /// This flag requires `--dev` **or** `--chain=...`,
    /// `--force-authoring` and `--alice` flags.
    #[clap(long = "instant-seal")]
    pub instant_seal: bool,
}

#[derive(Debug)]
pub struct RelayChainCli {
    /// The actual relay chain cli object.
    pub base: polkadot_cli::RunCmd,

    /// Optional chain id that should be passed to the relay chain.
    pub chain_id: Option<String>,

    /// The base path that should be used by the relay chain.
    pub base_path: Option<PathBuf>,
}

impl RelayChainCli {
    /// Parse the relay chain CLI parameters using the para chain `Configuration`.
    pub fn new<'a>(
        para_config: &sc_service::Configuration,
        relay_chain_args: impl Iterator<Item = &'a String>,
    ) -> Self {
        let extension = chain_spec::Extensions::try_get(&*para_config.chain_spec);
        let chain_id = extension.map(|e| e.relay_chain.clone());
        let base_path = para_config.base_path.path().join("polkadot");
        Self {
            base_path: Some(base_path),
            chain_id,
            base: clap::Parser::parse_from(relay_chain_args),
        }
    }
}
