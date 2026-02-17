use vortex_protocol::graph::{NodeDef, PortDef, DataType};
use crate::units::VortexUnit;

/// Native unit for sampling latent space.
pub struct KSampler;

impl VortexUnit for KSampler {
    fn definition(&self) -> NodeDef {
        NodeDef {
            type_id: "core.ksampler".into(),
            display_name: "K-Sampler".into(),
            category: "Sampling".into(),
            description: "Main sampling node using Kahn's scheduler for timing".into(),
            inputs: vec![
                PortDef {
                    name: "model".into(),
                    label: "MODEL".into(),
                    data_type: DataType::DataModel as i32,
                    required: true,
                    default_json: Vec::new(),
                    description: "Model weights".into(),
                },
                PortDef {
                    name: "positive".into(),
                    label: "Positive".into(),
                    data_type: DataType::DataConditioning as i32,
                    required: true,
                    default_json: Vec::new(),
                    description: "Positive prompt conditioning".into(),
                },
                PortDef {
                    name: "negative".into(),
                    label: "Negative".into(),
                    data_type: DataType::DataConditioning as i32,
                    required: true,
                    default_json: Vec::new(),
                    description: "Negative prompt conditioning".into(),
                },
                PortDef {
                    name: "latent".into(),
                    label: "Latent".into(),
                    data_type: DataType::DataLatent as i32,
                    required: true,
                    default_json: Vec::new(),
                    description: "Starting latent noise".into(),
                },
                PortDef {
                    name: "seed".into(),
                    label: "Seed".into(),
                    data_type: DataType::DataInt as i32,
                    required: false,
                    default_json: b"42".to_vec(),
                    description: "Random seed".into(),
                }
            ],
            outputs: vec![
                PortDef {
                    name: "latent".into(),
                    label: "LATENT".into(),
                    data_type: DataType::DataLatent as i32,
                    required: true,
                    default_json: Vec::new(),
                    description: "Sampled latent".into(),
                }
            ],
            author: "Vortex Core".into(),
            version: "1.0.0".into(),
        }
    }
}
