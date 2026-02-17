use vortex_protocol::graph::{NodeDef, PortDef, DataType};
use crate::units::VortexUnit;

/// Native unit for loading models.
pub struct ModelLoader;

impl VortexUnit for ModelLoader {
    fn definition(&self) -> NodeDef {
        NodeDef {
            type_id: "core.model_loader".into(),
            display_name: "Checkpoint Loader".into(),
            category: "Loaders".into(),
            description: "Loads a generative model (Stable Diffusion, etc.)".into(),
            inputs: vec![
                PortDef {
                    name: "model_name".into(),
                    label: "Model Name".into(),
                    data_type: DataType::DataString as i32,
                    required: true,
                    default_json: b"\"sd_v1.5.safetensors\"".to_vec(),
                    description: "Filename of the checkpoint to load".into(),
                }
            ],
            outputs: vec![
                PortDef {
                    name: "model".into(),
                    label: "MODEL".into(),
                    data_type: DataType::DataModel as i32,
                    required: true,
                    default_json: Vec::new(),
                    description: "Loaded model weights".into(),
                },
                PortDef {
                    name: "clip".into(),
                    label: "CLIP".into(),
                    data_type: DataType::DataClip as i32,
                    required: true,
                    default_json: Vec::new(),
                    description: "Text encoder".into(),
                },
                PortDef {
                    name: "vae".into(),
                    label: "VAE".into(),
                    data_type: DataType::DataVae as i32,
                    required: true,
                    default_json: Vec::new(),
                    description: "Variational Autoencoder".into(),
                }
            ],
            author: "Vortex Core".into(),
            version: "1.0.0".into(),
        }
    }
}
