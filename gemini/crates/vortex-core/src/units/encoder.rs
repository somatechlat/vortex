use vortex_protocol::graph::{NodeDef, PortDef, DataType};
use crate::units::VortexUnit;

/// Native unit for encoding text to conditioning.
pub struct TextEncoder;

impl VortexUnit for TextEncoder {
    fn definition(&self) -> NodeDef {
        NodeDef {
            type_id: "core.text_encoder".into(),
            display_name: "CLIP Text Encode".into(),
            category: "Conditioning".into(),
            description: "Encodes natural language into tensor conditioning".into(),
            inputs: vec![
                PortDef {
                    name: "clip".into(),
                    label: "CLIP".into(),
                    data_type: DataType::DataClip as i32,
                    required: true,
                    default_json: Vec::new(),
                    description: "Text encoder model".into(),
                },
                PortDef {
                    name: "text".into(),
                    label: "Text Prompt".into(),
                    data_type: DataType::DataString as i32,
                    required: true,
                    default_json: b"\"a cinematic photo of a robot\"".to_vec(),
                    description: "Textual description".into(),
                }
            ],
            outputs: vec![
                PortDef {
                    name: "conditioning".into(),
                    label: "CONDITIONING".into(),
                    data_type: DataType::DataConditioning as i32,
                    required: true,
                    default_json: Vec::new(),
                    description: "Conditioning tensor".into(),
                }
            ],
            author: "Vortex Core".into(),
            version: "1.0.0".into(),
        }
    }
}
