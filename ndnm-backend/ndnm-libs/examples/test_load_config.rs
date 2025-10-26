// Example: Load and validate the node-file-browser config
use ndnm_libs::load_config;

fn main() {
    let config_path = "nodes/node-file-browser/config.yaml";

    match load_config(config_path) {
        Ok(config) => {
            println!("✓ Successfully loaded config!");
            println!("  Node ID: {}", config.node_id_hash);
            println!("  Label: {}", config.label);
            println!("  Type: {}", config.node_type);
            println!("  Sections: {}", config.sections.len());
            println!("  Input Fields: {}", config.input_fields.len());

            for (i, section) in config.sections.iter().enumerate() {
                println!("\n  Section {}: {}", i + 1, section.section_name);
                println!("    Behavior: {:?}", section.behavior);
                println!("    Input: {} ({})", section.slot_template.input.name, section.slot_template.input.label);
                println!("    Output: {} ({})", section.slot_template.output.name, section.slot_template.output.label);
            }

            println!("\n  Input Fields:");
            for field in &config.input_fields {
                println!("    - {} ({:?})", field.label, field.field_type);
            }
        }
        Err(e) => {
            eprintln!("✗ Failed to load config: {}", e);
            std::process::exit(1);
        }
    }
}
