use clap::{Parser, Subcommand};
use formproof::{
    schema_fingerprint, verify, CompiledSchema, FormProofSchema, Proof, ProofPackage, Witness,
};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "formproof")]
#[command(
    about = "Tool hosts shouldn't have to see the refund amount to know it's ≤ $50 and a valid currency."
)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile a JSON Schema into proving/verifying keys
    Compile {
        /// Path to JSON Schema file
        #[arg(short, long)]
        schema: PathBuf,

        /// Output directory for compiled keys
        #[arg(short, long, default_value = ".")]
        output: PathBuf,
    },

    /// Generate a proof for a witness against a compiled schema
    Prove {
        /// Path to JSON Schema file
        #[arg(short, long)]
        schema: PathBuf,

        /// Path to proving key file
        #[arg(short, long)]
        proving_key: PathBuf,

        /// Path to witness JSON file
        #[arg(short, long)]
        witness: PathBuf,

        /// Output path for proof
        #[arg(short, long, default_value = "proof.bin")]
        output: PathBuf,
    },

    /// Verify a proof against a compiled schema
    Verify {
        /// Path to JSON Schema file
        #[arg(short, long)]
        schema: PathBuf,

        /// Path to verifying key file
        #[arg(short = 'k', long)]
        verifying_key: PathBuf,

        /// Path to proof file
        #[arg(short, long)]
        proof: PathBuf,

        /// Commitment (hex-encoded 32 bytes)
        #[arg(short, long)]
        commitment: String,
    },

    /// Show schema info and constraints
    Info {
        /// Path to JSON Schema file
        #[arg(short, long)]
        schema: PathBuf,
    },

    /// Build a portable proof package (bundles proof + commitment + schema fingerprint)
    PackageBuild {
        /// Path to JSON Schema file
        #[arg(short, long)]
        schema: PathBuf,

        /// Path to proving key file
        #[arg(short, long)]
        proving_key: PathBuf,

        /// Path to witness JSON file
        #[arg(short, long)]
        witness: PathBuf,

        /// Output path for package JSON
        #[arg(short, long, default_value = "proof_package.json")]
        output: PathBuf,

        /// Use compact JSON (no whitespace)
        #[arg(long)]
        compact: bool,
    },

    /// Verify a proof package against a compiled schema
    PackageVerify {
        /// Path to JSON Schema file
        #[arg(short, long)]
        schema: PathBuf,

        /// Path to verifying key file
        #[arg(short = 'k', long)]
        verifying_key: PathBuf,

        /// Path to proof package JSON file
        #[arg(short, long)]
        package: PathBuf,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compile { schema, output } => {
            println!("Loading schema from {:?}...", schema);
            let schema_json = fs::read_to_string(&schema)?;
            let parsed_schema = FormProofSchema::from_json(&schema_json)?;

            println!("Schema has {} properties", parsed_schema.properties.len());
            for prop in &parsed_schema.properties {
                let req = if prop.required { " (required)" } else { "" };
                println!("  - {}: {:?}{}", prop.name, prop.prop_type, req);
            }

            println!("\nCompiling circuit (this may take a moment)...");
            let compiled = CompiledSchema::compile(parsed_schema)?;

            fs::create_dir_all(&output)?;

            let pk_path = output.join("proving_key.bin");
            let vk_path = output.join("verifying_key.bin");
            let schema_path = output.join("schema.json");

            let pk_bytes = compiled.serialize_proving_key()?;
            let vk_bytes = compiled.serialize_verifying_key()?;

            fs::write(&pk_path, &pk_bytes)?;
            fs::write(&vk_path, &vk_bytes)?;
            fs::write(&schema_path, compiled.schema.to_json())?;

            println!("\nCompiled successfully!");
            println!("  Proving key:   {:?} ({} bytes)", pk_path, pk_bytes.len());
            println!("  Verifying key: {:?} ({} bytes)", vk_path, vk_bytes.len());
            println!("  Schema:        {:?}", schema_path);
        }

        Commands::Prove {
            schema,
            proving_key,
            witness: witness_path,
            output,
        } => {
            println!("Loading schema...");
            let schema_json = fs::read_to_string(&schema)?;
            let parsed_schema = FormProofSchema::from_json(&schema_json)?;

            println!("Loading proving key...");
            let pk_bytes = fs::read(&proving_key)?;
            let _vk_bytes: [u8; 0] = [];

            let dummy_circuit =
                formproof::circuit::FormProofCircuit::<ark_bn254::Fr>::new(parsed_schema.clone());
            let mut rng = ark_std::rand::rngs::OsRng;
            let (_, vk) = ark_groth16::Groth16::<ark_bn254::Bn254>::circuit_specific_setup(
                dummy_circuit,
                &mut rng,
            )?;
            let pvk = ark_groth16::Groth16::<ark_bn254::Bn254>::process_vk(&vk)?;

            let pk =
                ark_groth16::ProvingKey::<ark_bn254::Bn254>::deserialize_compressed(&pk_bytes[..])?;

            let compiled = CompiledSchema {
                schema: parsed_schema.clone(),
                proving_key: pk,
                verifying_key: pvk,
            };

            println!("Loading witness...");
            let witness_json = fs::read_to_string(&witness_path)?;
            let witness = parse_witness(&parsed_schema, &witness_json)?;

            println!("Generating proof...");
            let proof = Proof::create(&compiled, &witness)?;

            let proof_bytes = proof.serialize()?;
            fs::write(&output, &proof_bytes)?;

            let commitment_hex = hex::encode(proof.commitment);
            println!("\nProof generated successfully!");
            println!("  Proof:      {:?} ({} bytes)", output, proof_bytes.len());
            println!("  Commitment: {}", commitment_hex);
            println!("\nShare the proof and commitment with the verifier.");
        }

        Commands::Verify {
            schema,
            verifying_key,
            proof: proof_path,
            commitment,
        } => {
            println!("Loading schema...");
            let schema_json = fs::read_to_string(&schema)?;
            let parsed_schema = FormProofSchema::from_json(&schema_json)?;

            println!("Loading verifying key...");
            let vk_bytes = fs::read(&verifying_key)?;

            use ark_serialize::CanonicalDeserialize;
            let pvk =
                ark_groth16::PreparedVerifyingKey::<ark_bn254::Bn254>::deserialize_compressed(
                    &vk_bytes[..],
                )?;

            let dummy_pk = {
                let dummy_circuit = formproof::circuit::FormProofCircuit::<ark_bn254::Fr>::new(
                    parsed_schema.clone(),
                );
                let mut rng = ark_std::rand::rngs::OsRng;
                let (pk, _) = ark_groth16::Groth16::<ark_bn254::Bn254>::circuit_specific_setup(
                    dummy_circuit,
                    &mut rng,
                )?;
                pk
            };

            let compiled = CompiledSchema {
                schema: parsed_schema,
                proving_key: dummy_pk,
                verifying_key: pvk,
            };

            println!("Loading proof...");
            let proof_bytes = fs::read(&proof_path)?;

            let commitment_bytes: [u8; 32] = hex::decode(&commitment)?
                .try_into()
                .map_err(|_| "commitment must be exactly 32 bytes")?;

            let proof = Proof::deserialize(&proof_bytes, commitment_bytes)?;

            println!("Verifying...");
            match verify(&compiled, &proof) {
                Ok(true) => {
                    println!("\n✓ Proof is VALID");
                    println!("  The prover has demonstrated that their private data");
                    println!("  satisfies all schema constraints.");
                }
                Ok(false) => {
                    println!("\n✗ Proof is INVALID");
                    std::process::exit(1);
                }
                Err(e) => {
                    println!("\n✗ Verification error: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Info { schema } => {
            let schema_json = fs::read_to_string(&schema)?;
            let parsed_schema = FormProofSchema::from_json(&schema_json)?;

            println!("FormProof Schema Info");
            println!("=====================");
            println!("Properties: {}", parsed_schema.properties.len());
            println!(
                "Fingerprint: {}",
                hex::encode(schema_fingerprint(&parsed_schema))
            );
            println!();

            for prop in &parsed_schema.properties {
                let req = if prop.required { " [REQUIRED]" } else { "" };
                println!("• {}{}", prop.name, req);

                match &prop.prop_type {
                    formproof::PropertyType::U64 { minimum, maximum } => {
                        print!("  Type: integer");
                        if let Some(min) = minimum {
                            print!(", min={}", min);
                        }
                        if let Some(max) = maximum {
                            print!(", max={}", max);
                        }
                        println!();
                    }
                    formproof::PropertyType::Enum { variants } => {
                        println!("  Type: enum");
                        println!("  Values: {:?}", variants);
                    }
                    formproof::PropertyType::Bytes32 => {
                        println!("  Type: bytes32");
                    }
                    formproof::PropertyType::String { max_length } => {
                        println!("  Type: string, maxLength={}", max_length);
                    }
                }
            }
        }

        Commands::PackageBuild {
            schema,
            proving_key,
            witness: witness_path,
            output,
            compact,
        } => {
            println!("Loading schema...");
            let schema_json = fs::read_to_string(&schema)?;
            let parsed_schema = FormProofSchema::from_json(&schema_json)?;

            println!("Loading proving key...");
            let pk_bytes = fs::read(&proving_key)?;

            let dummy_circuit =
                formproof::circuit::FormProofCircuit::<ark_bn254::Fr>::new(parsed_schema.clone());
            let mut rng = ark_std::rand::rngs::OsRng;
            let (_, vk) = ark_groth16::Groth16::<ark_bn254::Bn254>::circuit_specific_setup(
                dummy_circuit,
                &mut rng,
            )?;
            let pvk = ark_groth16::Groth16::<ark_bn254::Bn254>::process_vk(&vk)?;

            let pk =
                ark_groth16::ProvingKey::<ark_bn254::Bn254>::deserialize_compressed(&pk_bytes[..])?;

            let compiled = CompiledSchema {
                schema: parsed_schema.clone(),
                proving_key: pk,
                verifying_key: pvk,
            };

            println!("Loading witness...");
            let witness_json = fs::read_to_string(&witness_path)?;
            let witness = parse_witness(&parsed_schema, &witness_json)?;

            println!("Generating proof and building package...");
            let package = ProofPackage::create(&compiled, &witness)?;

            let json_output = if compact {
                package.to_json_compact()?
            } else {
                package.to_json()?
            };

            fs::write(&output, &json_output)?;

            println!("\nPackage built successfully!");
            println!("  Output:      {:?} ({} bytes)", output, json_output.len());
            println!("  Commitment:  {}", package.commitment_hex);
            println!("  Fingerprint: {}", package.schema_fingerprint);
            println!("\nThe package contains everything needed for verification.");
            println!("Share this single file with the verifier.");
        }

        Commands::PackageVerify {
            schema,
            verifying_key,
            package: package_path,
        } => {
            println!("Loading schema...");
            let schema_json = fs::read_to_string(&schema)?;
            let parsed_schema = FormProofSchema::from_json(&schema_json)?;

            println!("Loading verifying key...");
            let vk_bytes = fs::read(&verifying_key)?;

            use ark_serialize::CanonicalDeserialize;
            let pvk =
                ark_groth16::PreparedVerifyingKey::<ark_bn254::Bn254>::deserialize_compressed(
                    &vk_bytes[..],
                )?;

            let dummy_pk = {
                let dummy_circuit = formproof::circuit::FormProofCircuit::<ark_bn254::Fr>::new(
                    parsed_schema.clone(),
                );
                let mut rng = ark_std::rand::rngs::OsRng;
                let (pk, _) = ark_groth16::Groth16::<ark_bn254::Bn254>::circuit_specific_setup(
                    dummy_circuit,
                    &mut rng,
                )?;
                pk
            };

            let compiled = CompiledSchema {
                schema: parsed_schema,
                proving_key: dummy_pk,
                verifying_key: pvk,
            };

            println!("Loading package...");
            let package_json = fs::read_to_string(&package_path)?;
            let package = ProofPackage::from_json(&package_json)?;

            println!("Verifying package...");
            println!("  Commitment:  {}", package.commitment_hex);
            println!("  Fingerprint: {}", package.schema_fingerprint);

            match package.verify(&compiled) {
                Ok(()) => {
                    println!("\n✓ Package is VALID");
                    println!("  The prover has demonstrated that their private data");
                    println!("  satisfies all schema constraints.");
                }
                Err(e) => {
                    println!("\n✗ Package verification failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}

fn parse_witness(
    schema: &FormProofSchema,
    json: &str,
) -> Result<Witness, Box<dyn std::error::Error>> {
    let value: serde_json::Value = serde_json::from_str(json)?;
    let obj = value.as_object().ok_or("witness must be a JSON object")?;

    let mut witness = Witness::new();

    for prop in &schema.properties {
        if let Some(val) = obj.get(&prop.name) {
            match &prop.prop_type {
                formproof::PropertyType::U64 { .. } => {
                    let n = val
                        .as_u64()
                        .ok_or_else(|| format!("{} must be an integer", prop.name))?;
                    witness.set_u64(&prop.name, n);
                }
                formproof::PropertyType::Enum { .. } => {
                    let s = val
                        .as_str()
                        .ok_or_else(|| format!("{} must be a string", prop.name))?;
                    witness.set_enum(&prop.name, s);
                }
                formproof::PropertyType::Bytes32 => {
                    let s = val
                        .as_str()
                        .ok_or_else(|| format!("{} must be a hex string", prop.name))?;
                    let bytes = hex::decode(s)?;
                    let arr: [u8; 32] = bytes
                        .try_into()
                        .map_err(|_| format!("{} must be 32 bytes", prop.name))?;
                    witness.set_bytes32(&prop.name, arr);
                }
                formproof::PropertyType::String { .. } => {
                    let s = val
                        .as_str()
                        .ok_or_else(|| format!("{} must be a string", prop.name))?;
                    witness.set_string(&prop.name, s);
                }
            }
        } else {
            witness.set_absent(&prop.name);
        }
    }

    Ok(witness)
}

use ark_serialize::CanonicalDeserialize;
use ark_snark::SNARK;
