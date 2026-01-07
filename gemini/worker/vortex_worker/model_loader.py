"""Model Loader for VORTEX Worker.

Provides HuggingFace Hub integration for loading models, including:
- Diffusers pipelines (SD, SDXL, SD Turbo)
- Safetensors checkpoints
- Model caching and management
"""

import logging
import os
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

logger = logging.getLogger(__name__)


# ═══════════════════════════════════════════════════════════════
#                    MODEL CATALOG
# ═══════════════════════════════════════════════════════════════


@dataclass
class ModelInfo:
    """Information about a model in the catalog."""

    id: str  # HuggingFace repo ID or local path
    name: str  # Display name
    type: str  # checkpoint, diffusers, lora, vae, etc.
    base: str | None = None  # Base model (for LoRAs)
    size_mb: int = 0  # Approximate size
    tags: list[str] = field(default_factory=list)
    local_path: str | None = None  # Cached local path


# Pre-defined model catalog (free/open models)
MODEL_CATALOG: dict[str, ModelInfo] = {
    # Stable Diffusion 1.5
    "sd15": ModelInfo(
        id="runwayml/stable-diffusion-v1-5",
        name="Stable Diffusion 1.5",
        type="diffusers",
        size_mb=4000,
        tags=["sd15", "base", "free"],
    ),
    # SDXL Turbo (fast, free)
    "sdxl-turbo": ModelInfo(
        id="stabilityai/sdxl-turbo",
        name="SDXL Turbo",
        type="diffusers",
        size_mb=6500,
        tags=["sdxl", "turbo", "fast", "free"],
    ),
    # SD Turbo (even faster)
    "sd-turbo": ModelInfo(
        id="stabilityai/sd-turbo",
        name="SD Turbo",
        type="diffusers",
        size_mb=3500,
        tags=["sd21", "turbo", "fast", "free"],
    ),
    # LCM LoRA for fast inference
    "lcm-lora-sdxl": ModelInfo(
        id="latent-consistency/lcm-lora-sdxl",
        name="LCM LoRA (SDXL)",
        type="lora",
        base="sdxl",
        size_mb=400,
        tags=["lcm", "lora", "fast", "free"],
    ),
    # Realistic Vision
    "realistic-vision": ModelInfo(
        id="SG161222/Realistic_Vision_V5.1_noVAE",
        name="Realistic Vision V5.1",
        type="diffusers",
        size_mb=4000,
        tags=["sd15", "realistic", "free"],
    ),
}


# ═══════════════════════════════════════════════════════════════
#                    MODEL LOADER
# ═══════════════════════════════════════════════════════════════


class ModelLoader:
    """Loads and caches models from HuggingFace Hub."""

    def __init__(self, cache_dir: str | None = None):
        self.cache_dir = Path(
            cache_dir
            or os.environ.get("HF_HOME", Path.home() / ".cache" / "huggingface")
        )
        self.cache_dir.mkdir(parents=True, exist_ok=True)

        self.loaded_models: dict[str, Any] = {}
        self.catalog = MODEL_CATALOG.copy()

        logger.info(f"ModelLoader initialized, cache: {self.cache_dir}")

    def list_models(self, tag: str | None = None) -> list[ModelInfo]:
        """List available models, optionally filtered by tag."""
        models = list(self.catalog.values())
        if tag:
            models = [m for m in models if tag in m.tags]
        return models

    def get_model_info(self, model_id: str) -> ModelInfo | None:
        """Get info for a model by ID."""
        return self.catalog.get(model_id)

    def is_loaded(self, model_id: str) -> bool:
        """Check if a model is already loaded in memory."""
        return model_id in self.loaded_models

    def load_pipeline(
        self,
        model_id: str,
        device: str = "cuda",
        dtype: str = "float16",
        variant: str | None = "fp16",
    ) -> Any:
        """Load a diffusers pipeline.

        Args:
            model_id: Model key from catalog or HuggingFace repo ID
            device: Target device (cuda, cpu, mps)
            dtype: Model dtype (float16, float32, bfloat16)
            variant: Weight variant (fp16, etc.)

        Returns:
            Loaded diffusers pipeline
        """
        # Check cache first
        if model_id in self.loaded_models:
            logger.info(f"Using cached model: {model_id}")
            return self.loaded_models[model_id]

        # Get model info
        info = self.catalog.get(model_id)
        repo_id = info.id if info else model_id

        logger.info(f"Loading model: {repo_id} to {device}")

        try:
            import torch
            from diffusers import AutoPipelineForText2Image

            # Map dtype string
            dtype_map = {
                "float16": torch.float16,
                "float32": torch.float32,
                "bfloat16": torch.bfloat16,
            }
            torch_dtype = dtype_map.get(dtype, torch.float16)

            # Load pipeline
            pipe = AutoPipelineForText2Image.from_pretrained(
                repo_id,
                torch_dtype=torch_dtype,
                variant=variant,
                cache_dir=str(self.cache_dir),
            )

            # Move to device
            if device == "cuda" and torch.cuda.is_available():
                pipe = pipe.to("cuda")
            elif device == "mps" and hasattr(torch.backends, "mps"):
                pipe = pipe.to("mps")
            else:
                pipe = pipe.to("cpu")

            # Cache the loaded model
            self.loaded_models[model_id] = pipe

            logger.info(f"Model loaded successfully: {model_id}")
            return pipe

        except ImportError as e:
            logger.error(f"Missing dependency: {e}")
            raise
        except Exception as e:
            logger.error(f"Failed to load model {model_id}: {e}")
            raise

    def load_checkpoint(
        self,
        path: str,
        device: str = "cuda",
    ) -> dict[str, Any]:
        """Load a safetensors checkpoint.

        Args:
            path: Path to .safetensors file
            device: Target device

        Returns:
            State dict
        """
        try:
            import torch
            from safetensors import safe_open

            tensors = {}
            with safe_open(path, framework="pt", device=device) as f:
                for key in f.keys():
                    tensors[key] = f.get_tensor(key)

            logger.info(f"Loaded checkpoint: {path} ({len(tensors)} tensors)")
            return tensors

        except ImportError:
            logger.error("safetensors not installed")
            raise

    def unload(self, model_id: str) -> bool:
        """Unload a model to free memory."""
        if model_id in self.loaded_models:
            del self.loaded_models[model_id]

            # Force garbage collection
            import gc

            gc.collect()

            try:
                import torch

                if torch.cuda.is_available():
                    torch.cuda.empty_cache()
            except ImportError:
                pass

            logger.info(f"Unloaded model: {model_id}")
            return True
        return False

    def unload_all(self) -> int:
        """Unload all models."""
        count = len(self.loaded_models)
        self.loaded_models.clear()

        import gc

        gc.collect()

        try:
            import torch

            if torch.cuda.is_available():
                torch.cuda.empty_cache()
        except ImportError:
            pass

        logger.info(f"Unloaded {count} models")
        return count


# ═══════════════════════════════════════════════════════════════
#                    INFERENCE HELPER
# ═══════════════════════════════════════════════════════════════


def generate_image(
    pipe: Any,
    prompt: str,
    negative_prompt: str = "",
    width: int = 512,
    height: int = 512,
    steps: int = 20,
    cfg: float = 7.0,
    seed: int | None = None,
) -> Any:
    """Generate an image using a diffusers pipeline.

    Args:
        pipe: Loaded diffusers pipeline
        prompt: Text prompt
        negative_prompt: Negative prompt
        width: Output width
        height: Output height
        steps: Inference steps
        cfg: Classifier-free guidance scale
        seed: Random seed

    Returns:
        PIL Image or list of images
    """
    import torch

    generator = None
    if seed is not None:
        device = pipe.device if hasattr(pipe, "device") else "cpu"
        generator = torch.Generator(device=device).manual_seed(seed)

    logger.info(f"Generating: {prompt[:50]}... ({width}x{height}, {steps} steps)")

    result = pipe(
        prompt=prompt,
        negative_prompt=negative_prompt or None,
        width=width,
        height=height,
        num_inference_steps=steps,
        guidance_scale=cfg,
        generator=generator,
    )

    return result.images[0] if result.images else None


# Global loader instance
_loader: ModelLoader | None = None


def get_loader() -> ModelLoader:
    """Get the global model loader instance."""
    global _loader
    if _loader is None:
        _loader = ModelLoader()
    return _loader
