use anyhow::{anyhow, Context, Result};
use clap::Arg;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::BufReader;

mod diagnostics;
mod dirgraphsvg;
mod gsn;
mod render;
mod yaml_fix;

use diagnostics::Diagnostics;
use dirgraphsvg::escape_text;
use gsn::{GsnDocumentNode, GsnNode, Module, ModuleInformation};

const MODULE_INFORMATION_NODE: &str = "module";

///
/// Main entry point.
///
///
fn main() -> Result<()> {
    let app = clap::command!()
        .arg(
            Arg::new("INPUT")
                .help("Sets the input file(s) to use.")
                .multiple_occurrences(true)
                .required(true),
        )
        .arg(
            Arg::new("CHECKONLY")
                .help("Only check the input file(s), but do not output graphs.")
                .short('c')
                .long("check")
                .help_heading("CHECKS"),
        )
        .arg(
            Arg::new("EXCLUDED_MODULE")
                .help("Exclude this module from reference checks.")
                .short('x')
                .long("exclude")
                .multiple_occurrences(true)
                .takes_value(true)
                .help_heading("CHECKS"),
        )
        .arg(
            Arg::new("NO_ARGUMENT_VIEW")
                .help("Do not output of argument view for provided input files.")
                .short('N')
                .long("no-arg")
                .help_heading("OUTPUT"),
        )
        .arg(
            Arg::new("COMPLETE_VIEW")
                .help("Output the complete view to <COMPLETE_VIEW>.")
                .short('f')
                .long("full")
                .takes_value(true)
                .conflicts_with_all(&["CHECKONLY", "NO_COMPLETE_VIEW"])
                .help_heading("OUTPUT"),
        )
        .arg(
            Arg::new("NO_COMPLETE_VIEW")
                .help("Do not output the complete view.")
                .short('F')
                .long("no-full")
                .takes_value(false)
                .conflicts_with("COMPLETE_VIEW")
                .help_heading("OUTPUT"),
        )
        .arg(
            Arg::new("ARCHITECTURE_VIEW")
                .help("Output the architecture view to <ARCHITECTURE_VIEW>.")
                .short('a')
                .long("arch")
                .takes_value(true)
                .conflicts_with_all(&["CHECKONLY", "NO_ARCHITECTURE_VIEW"])
                .help_heading("OUTPUT"),
        )
        .arg(
            Arg::new("NO_ARCHITECTURE_VIEW")
                .help("Do not output the architecture view.")
                .short('A')
                .long("no-arch")
                .takes_value(false)
                .conflicts_with("ARCHITECTURE_VIEW")
                .help_heading("OUTPUT"),
        )
        .arg(
            Arg::new("EVIDENCES")
                .help("Output list of all evidences to <EVIDENCES>.")
                .short('e')
                .long("evidences")
                .takes_value(true)
                .multiple_occurrences(false)
                .conflicts_with_all(&["CHECKONLY", "NO_EVIDENCES"])
                .help_heading("OUTPUT"),
        )
        .arg(
            Arg::new("NO_EVIDENCES")
                .help("Do not output list of all evidences.")
                .short('E')
                .long("no-evidences")
                .takes_value(false)
                .multiple_occurrences(false)
                .conflicts_with("EVIDENCES")
                .help_heading("OUTPUT"),
        )
        .arg(
            Arg::new("LAYERS")
                .help("Output additional layer. Can be used multiple times.")
                .short('l')
                .long("layer")
                .takes_value(true)
                .multiple_occurrences(true)
                .use_value_delimiter(true)
                .conflicts_with("CHECKONLY")
                .help_heading("OUTPUT MODIFICATION"),
        )
        .arg(
            Arg::new("STYLESHEETS")
                .help("Links a stylesheet in SVG output. Can be used multiple times.")
                .short('s')
                .long("stylesheet")
                .takes_value(true)
                .multiple_occurrences(true)
                .conflicts_with("CHECKONLY")
                .help_heading("OUTPUT MODIFICATION"),
        )
        .arg(
            Arg::new("EMBED_CSS")
                .help("Embed stylehseets instead of linking them.")
                .short('t')
                .long("embed-css")
                .takes_value(false)
                .multiple_occurrences(false)
                .conflicts_with("CHECKONLY")
                .help_heading("OUTPUT MODIFICATION"),
        )
        // .arg(
        //     Arg::new("MASK_MODULE")
        //         .help("Do not unroll this module in the complete view.")
        //         .short('m')
        //         .long("mask")
        //         .multiple_occurrences(true)
        //         .takes_value(true)
        //         .requires("COMPLETE_VIEW")
        //         .help_heading("OUTPUT MODIFICATION"),
        // )
        .arg(
            Arg::new("NO_LEGEND")
                .help("Do not output a legend based on module information.")
                .short('G')
                .long("no-legend")
                .conflicts_with("CHECKONLY")
                .help_heading("OUTPUT MODIFICATION"),
        )
        .arg(
            Arg::new("FULL_LEGEND")
                .help("Output a legend based on all module information.")
                .short('g')
                .long("full-legend")
                .conflicts_with("CHECKONLY")
                .help_heading("OUTPUT MODIFICATION"),
        );
    let matches = app.get_matches();
    let mut diags = Diagnostics::default();
    let inputs: Vec<&str> = matches.values_of("INPUT").unwrap().collect();
    let mut nodes = BTreeMap::<String, GsnNode>::new();
    let layers = matches
        .values_of("LAYERS")
        .map(|x| x.collect::<Vec<&str>>());
    let stylesheets = matches
        .values_of("STYLESHEETS")
        .map(|x| x.collect::<Vec<&str>>());
    let embed_stylesheets = matches.is_present("EMBED_CSS");
    let excluded_modules = matches
        .values_of("EXCLUDED_MODULE")
        .map(|x| x.collect::<Vec<&str>>());
    // Module name to module mapping
    let mut modules: HashMap<String, Module> = HashMap::new();

    // Read input
    read_inputs(&inputs, &mut nodes, &mut modules, &mut diags)?;

    // Validate
    validate_and_check(&mut nodes, &modules, &mut diags, excluded_modules, &layers);

    if diags.errors == 0 && !matches.is_present("CHECKONLY") {
        // Output argument view
        print_outputs(
            &matches,
            nodes,
            &modules,
            &layers,
            stylesheets,
            embed_stylesheets,
        )?;
    }
    // Output diagnostic messages
    output_messages(&diags)
}

///
/// Read inputs
///
///
fn read_inputs(
    inputs: &[&str],
    nodes: &mut BTreeMap<String, GsnNode>,
    modules: &mut HashMap<String, Module>,
    diags: &mut Diagnostics,
) -> Result<()> {
    for input in inputs {
        let reader =
            BufReader::new(File::open(&input).context(format!("Failed to open file {}", input))?);

        let mut n: BTreeMap<String, GsnDocumentNode> = serde_yaml::from_reader(reader)
            .map(|n: yaml_fix::YamlFixMap<String, GsnDocumentNode>| n.into_inner())
            .map_err(|e| {
                anyhow!(format!(
                    "No valid GSN element can be found starting from line {}",
                    e.location().unwrap().line()
                ))
            })
            .context(format!("Failed to parse YAML from file {}", input))?;
        let mut meta: Option<ModuleInformation> = match n.remove_entry(MODULE_INFORMATION_NODE) {
            Some((_, GsnDocumentNode::ModuleInformation(x))) => Some(x),
            _ => None,
        };
        // Add filename and module name to module list
        let module = if let Some(m) = &meta {
            m.name.to_owned()
        } else {
            let module_name = escape_text(input);
            meta = Some(ModuleInformation {
                name: module_name.to_owned(),
                brief: None,
                extends: None,
                additional: BTreeMap::new(),
            });
            module_name
        };

        if let std::collections::hash_map::Entry::Vacant(e) = modules.entry(module.to_owned()) {
            e.insert(Module {
                filename: input.to_owned().to_owned(),
                meta,
            });
        } else {
            diags.add_error(
                Some(&module),
                format!(
                    "C06: Module name {} in {} was already present in {}.",
                    module,
                    input,
                    modules.get(&module).unwrap().filename
                ),
            );
        }

        // Check for duplicates, since they might be in separate files.
        let node_names: Vec<String> = n.keys().cloned().collect();
        for node_name in node_names {
            if let Some((k, v)) = n.remove_entry(&node_name) {
                if let std::collections::btree_map::Entry::Vacant(e) = nodes.entry(k.to_owned()) {
                    match v {
                        GsnDocumentNode::GsnNode(mut x) => {
                            // Remember module for node
                            x.module = module.to_owned();
                            e.insert(x);
                        }
                        _ => unreachable!(), // There can be only one MetaNode
                    }
                } else {
                    diags.add_error(
                        Some(&module),
                        format!(
                            "C07: Element {} in {} was already present in {}.",
                            k,
                            input,
                            nodes.get(&k).unwrap().module
                        ),
                    );
                    break;
                }
            }
        }
    }
    if nodes.is_empty() {
        Err(anyhow!("No input elements found."))
    } else {
        Ok(())
    }
}

///
/// Validate and check modules
///
///
///
///
fn validate_and_check(
    nodes: &mut BTreeMap<String, GsnNode>,
    modules: &HashMap<String, Module>,
    diags: &mut Diagnostics,
    excluded_modules: Option<Vec<&str>>,
    layers: &Option<Vec<&str>>,
) {
    for (module_name, module_info) in modules {
        // Validation for well-formedness is done unconditionally.
        gsn::validation::validate_module(diags, module_name, module_info, nodes);
        if diags.errors > 0 {
            break;
        }
    }
    if diags.errors == 0 {
        gsn::extend_modules(diags, nodes, modules);
        gsn::check::check_nodes(diags, nodes, excluded_modules);
        if let Some(lays) = &layers {
            gsn::check::check_layers(diags, nodes, lays);
        }
    }
}

///
/// Print outputs
///
///
///
///
fn print_outputs(
    matches: &clap::ArgMatches,
    nodes: BTreeMap<String, GsnNode>,
    modules: &HashMap<String, Module>,
    layers: &Option<Vec<&str>>,
    stylesheets: Option<Vec<&str>>,
    embed_stylesheets: bool,
) -> Result<()> {
    if !matches.is_present("NO_ARGUMENT_VIEW") {
        for (module_name, module) in modules {
            let mut pbuf = std::path::PathBuf::from(&module.filename);
            pbuf.set_extension("svg");
            let output_filename = pbuf.as_path();
            let mut output_file = Box::new(File::create(output_filename).context(format!(
                "Failed to open output file {}",
                output_filename.display()
            ))?) as Box<dyn std::io::Write>;
            render::render_argument(
                &mut output_file,
                matches,
                module_name,
                modules,
                &nodes,
                stylesheets
                    .iter()
                    .flatten()
                    .map(|x| Some(x.to_owned()))
                    .collect(),
                embed_stylesheets,
            )?;
        }
    }
    if modules.len() > 1 {
        if !matches.is_present("NO_ARCHITECTURE_VIEW") {
            let mut pbuf = std::path::PathBuf::from(&modules.iter().next().unwrap().1.filename);
            pbuf.set_file_name("architecture.svg");
            let output_filename = matches
                .value_of("ARCHITECTURE_VIEW")
                .or_else(|| pbuf.to_str())
                .unwrap();
            let mut output_file = File::create(output_filename)
                .context(format!("Failed to open output file {}", output_filename))?;
            let deps = crate::gsn::calculate_module_dependencies(&nodes);
            render::render_architecture(
                &mut output_file,
                modules,
                deps,
                stylesheets
                    .iter()
                    .flatten()
                    .map(|x| Some(x.to_owned()))
                    .collect(),
                embed_stylesheets,
            )?;
        }
        if !matches.is_present("NO_COMPLETE_VIEW") {
            let mut pbuf = std::path::PathBuf::from(&modules.iter().next().unwrap().1.filename);
            pbuf.set_file_name("complete.svg");
            let output_filename = matches
                .value_of("COMPLETE_VIEW")
                .or_else(|| pbuf.to_str())
                .unwrap();
            let mut output_file = File::create(output_filename)
                .context(format!("Failed to open output file {}", output_filename))?;
            render::render_complete(
                &mut output_file,
                matches,
                &nodes,
                stylesheets,
                embed_stylesheets,
            )?;
        }
    }
    if !matches.is_present("NO_EVIDENCES") {
        let mut pbuf = std::path::PathBuf::from(&modules.iter().next().unwrap().1.filename);
        pbuf.set_file_name("evidences.md");
        let output_filename = matches
            .value_of("EVIDENCES")
            .or_else(|| pbuf.to_str())
            .unwrap();
        let mut output_file = File::create(output_filename)
            .context(format!("Failed to open output file {}", output_filename))?;
        render::render_evidences(&mut output_file, &nodes, layers)?;
    }
    Ok(())
}

///
/// Render to dot-file if not only validation is active.
/// Output summary of warnings and errors.
///
fn output_messages(diags: &Diagnostics) -> Result<()> {
    for msg in &diags.messages {
        eprintln!("{}", msg);
    }
    if diags.errors == 0 {
        if diags.warnings > 0 {
            eprintln!("Warning: {} warnings detected.", diags.warnings);
        }
        Ok(())
    } else {
        Err(anyhow!(
            "{} errors and {} warnings detected.",
            diags.errors,
            diags.warnings
        ))
    }
}

#[cfg(test)]
mod test {
    use crate::diagnostics::Diagnostics;

    #[test]
    fn check_output_messages_errors() {
        let d = Diagnostics {
            warnings: 2,
            errors: 3,
            ..Default::default()
        };
        let res = crate::output_messages(&d);
        assert!(res.is_err());
        assert_eq!(
            format!("{:?}", res),
            "Err(3 errors and 2 warnings detected.)"
        );
    }

    #[test]
    fn check_output_messages_warnings() {
        let d = Diagnostics {
            warnings: 5,
            errors: 0,
            ..Default::default()
        };
        let res = crate::output_messages(&d);
        assert!(res.is_ok());
        assert_eq!(format!("{:?}", res), "Ok(())");
    }

    #[test]
    fn check_output_messages_no() {
        let d = Diagnostics {
            warnings: 0,
            errors: 0,
            ..Default::default()
        };
        let res = crate::output_messages(&d);
        assert!(res.is_ok());
        assert_eq!(format!("{:?}", res), "Ok(())");
    }
}
