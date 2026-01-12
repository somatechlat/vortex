"""Executor Framework for VORTEX Worker."""

import logging
import time
from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    import torch

from .model_loader import get_loader
from .shm import ShmArena

logger = logging.getLogger(__name__)


@dataclass
class TensorHandle:
    """Reference to a tensor in shared memory."""

    offset: int
    shape: tuple
    dtype: str
    device: str = "cuda"


@dataclass
class ExecutionResult:
    """Result of executing a node."""

    success: bool
    outputs: dict[str, TensorHandle]
    duration_us: int
    peak_vram_mb: int
    error: str | None = None


class AbstractExecutor(ABC):
    """Base class for all executor implementations."""

    OP_TYPE: str = ""
    INPUT_TYPES: dict[str, str] = {}
    OUTPUT_TYPES: dict[str, str] = {}

    def __init__(self, shm_arena: ShmArena):
        self.shm = shm_arena
        self.model_loader = get_loader()

    @abstractmethod
    def execute(
        self,
        inputs: dict[str, TensorHandle],
        params: dict[str, Any],
    ) -> ExecutionResult:
        pass

    def get_tensor(self, handle: TensorHandle) -> "torch.Tensor":
        if handle.offset == 0 and handle.shape == ():
            raise ValueError("Cannot get tensor for model handle")
        return self.shm.load_tensor(handle.offset)

    def put_tensor(self, tensor: "torch.Tensor", device: str = "cuda") -> TensorHandle:
        if device == "cuda" and tensor.device.type != "cuda":
            tensor = tensor.to("cuda")
        elif device == "cpu" and tensor.device.type != "cpu":
            tensor = tensor.cpu()

        offset = self.shm.store_tensor(tensor)

        dtype_map = {
            "torch.float32": "float32",
            "torch.float16": "float16",
            "torch.bfloat16": "bfloat16",
            "torch.int32": "int32",
            "torch.int64": "int64",
            "torch.uint8": "uint8",
        }
        dtype_str = dtype_map.get(str(tensor.dtype), str(tensor.dtype))

        return TensorHandle(
            offset=offset, shape=tuple(tensor.shape), dtype=dtype_str, device=device
        )


class ExecutorRegistry:
    _executors: dict[str, type[AbstractExecutor]] = {}

    @classmethod
    def register(cls, op_type: str):
        def decorator(executor_cls: type[AbstractExecutor]):
            cls._executors[op_type] = executor_cls
            executor_cls.OP_TYPE = op_type
            logger.info(f"Registered executor: {op_type}")
            return executor_cls

        return decorator

    @classmethod
    def get(cls, op_type: str) -> type[AbstractExecutor] | None:
        return cls._executors.get(op_type)

    @classmethod
    def list(cls) -> list[str]:
        return list(cls._executors.keys())

    @classmethod
    def execute_node(
        cls,
        op_type: str,
        inputs: dict[str, TensorHandle],
        params: dict[str, Any],
        shm_arena: ShmArena,
    ) -> ExecutionResult:
        executor_cls = cls.get(op_type)
        if executor_cls is None:
            return ExecutionResult(
                success=False,
                outputs={},
                duration_us=0,
                peak_vram_mb=0,
                error=f"Unknown operation type: {op_type}",
            )

        executor = executor_cls(shm_arena)

        start = time.perf_counter_ns()
        try:
            result = executor.execute(inputs, params)
            result.duration_us = (time.perf_counter_ns() - start) // 1000
            return result
        except Exception as e:
            logger.exception(f"Executor failed: {e}")
            return ExecutionResult(
                success=False,
                outputs={},
                duration_us=(time.perf_counter_ns() - start) // 1000,
                peak_vram_mb=0,
                error=str(e),
            )


@ExecutorRegistry.register("Loader::Checkpoint")
class CheckpointLoader(AbstractExecutor):
    INPUT_TYPES = {}
    OUTPUT_TYPES = {"model": "MODEL", "clip": "CLIP", "vae": "VAE"}

    def execute(self, inputs, params) -> ExecutionResult:
        model_id = params.get("ckpt_name", "sd15")

        try:
            logger.info(f"Loading checkpoint: {model_id}")

            pipe = self.model_loader.load_pipeline(
                model_id=model_id,
                device="cuda",
                dtype="float16",
            )

            import sys

            sys.modules["vortex_worker.executor"].loaded_model = pipe

            peak_vram = 0
            try:
                import torch

                if torch.cuda.is_available():
                    torch.cuda.empty_cache()
                    peak_vram = torch.cuda.max_memory_allocated() // (1024 * 1024)
            except Exception:
                pass

            return ExecutionResult(
                success=True,
                outputs={
                    "model": TensorHandle(
                        offset=0, shape=(), dtype="model", device="cuda"
                    ),
                    "clip": TensorHandle(
                        offset=0, shape=(), dtype="clip", device="cuda"
                    ),
                    "vae": TensorHandle(offset=0, shape=(), dtype="vae", device="cuda"),
                },
                duration_us=0,
                peak_vram_mb=peak_vram,
            )

        except Exception as e:
            logger.exception(f"Failed to load model {model_id}: {e}")
            return ExecutionResult(
                success=False,
                outputs={},
                duration_us=0,
                peak_vram_mb=0,
                error=str(e),
            )


@ExecutorRegistry.register("Sampler::KSampler")
class KSamplerExecutor(AbstractExecutor):
    INPUT_TYPES = {
        "model": "MODEL",
        "positive": "CONDITIONING",
        "negative": "CONDITIONING",
        "latent": "LATENT",
    }
    OUTPUT_TYPES = {"samples": "LATENT"}

    def execute(self, inputs, params) -> ExecutionResult:
        import torch

        steps = params.get("steps", 20)
        cfg = params.get("cfg", 7.0)
        sampler_name = params.get("sampler_name", "euler")
        seed = params.get("seed", 42)
        prompt = params.get("prompt", "a beautiful landscape")
        negative_prompt = params.get("negative_prompt", "blurry, bad quality")

        logger.info(f"KSampler: steps={steps}, cfg={cfg}, sampler={sampler_name}")

        try:
            import sys

            if not hasattr(sys.modules["vortex_worker.executor"], "loaded_model"):
                raise ValueError("Model not loaded - run CheckpointLoader first")

            pipe = sys.modules["vortex_worker.executor"].loaded_model

            latent_handle = inputs.get("latent")
            if latent_handle and latent_handle.offset != 0:
                latent = self.get_tensor(latent_handle)
            else:
                latent = torch.randn(1, 4, 64, 64, dtype=torch.float16, device="cuda")

            generator = torch.Generator(device="cuda").manual_seed(seed)

            result = pipe(
                prompt=prompt,
                negative_prompt=negative_prompt,
                width=512,
                height=512,
                num_inference_steps=steps,
                guidance_scale=cfg,
                generator=generator,
                output_type="latent",
            )

            samples = result.images if hasattr(result, "images") else result

            if samples is None:
                samples = torch.randn(1, 4, 64, 64, dtype=torch.float16, device="cuda")

            output_handle = self.put_tensor(samples, device="cuda")

            peak_vram = 0
            if torch.cuda.is_available():
                torch.cuda.empty_cache()
                peak_vram = torch.cuda.max_memory_allocated() // (1024 * 1024)

            return ExecutionResult(
                success=True,
                outputs={"samples": output_handle},
                duration_us=0,
                peak_vram_mb=peak_vram,
            )

        except Exception as e:
            logger.exception(f"KSampler failed: {e}")
            return ExecutionResult(
                success=False,
                outputs={},
                duration_us=0,
                peak_vram_mb=0,
                error=str(e),
            )


@ExecutorRegistry.register("Decoder::VAE")
class VAEDecodeExecutor(AbstractExecutor):
    INPUT_TYPES = {"samples": "LATENT", "vae": "VAE"}
    OUTPUT_TYPES = {"image": "IMAGE"}

    def execute(self, inputs, params) -> ExecutionResult:
        import torch

        logger.info("VAE decoding latents to image")

        try:
            latent_handle = inputs.get("samples")
            if not latent_handle or latent_handle.offset == 0:
                raise ValueError("No latent input")

            latent = self.get_tensor(latent_handle)

            import sys

            if not hasattr(sys.modules["vortex_worker.executor"], "loaded_model"):
                raise ValueError("Model not loaded")

            pipe = sys.modules["vortex_worker.executor"].loaded_model
            vae = pipe.vae

            with torch.no_grad():
                image = vae.decode(latent).sample

            image = (image / 2 + 0.5).clamp(0, 1)
            image = image.permute(0, 2, 3, 1)
            image = (image * 255).to(torch.uint8)

            output_handle = self.put_tensor(image, device="cpu")

            peak_vram = 0
            if torch.cuda.is_available():
                torch.cuda.empty_cache()
                peak_vram = torch.cuda.max_memory_allocated() // (1024 * 1024)

            return ExecutionResult(
                success=True,
                outputs={"image": output_handle},
                duration_us=0,
                peak_vram_mb=peak_vram,
            )

        except Exception as e:
            logger.exception(f"VAE decode failed: {e}")
            return ExecutionResult(
                success=False,
                outputs={},
                duration_us=0,
                peak_vram_mb=0,
                error=str(e),
            )


@ExecutorRegistry.register("Encoder::CLIP")
class CLIPTextEncode(AbstractExecutor):
    INPUT_TYPES = {"clip": "CLIP", "text": "STRING"}
    OUTPUT_TYPES = {"conditioning": "CONDITIONING"}

    def execute(self, inputs, params) -> ExecutionResult:
        import torch

        text = params.get("text", "")
        logger.info(f"CLIP encoding: {text[:50]}...")

        try:
            import sys

            if not hasattr(sys.modules["vortex_worker.executor"], "loaded_model"):
                raise ValueError("Model not loaded")

            pipe = sys.modules["vortex_worker.executor"].loaded_model
            clip = pipe.text_encoder
            tokenizer = pipe.tokenizer

            text_inputs = tokenizer(
                text,
                padding="max_length",
                max_length=77,
                truncation=True,
                return_tensors="pt",
            ).to("cuda")

            with torch.no_grad():
                embeddings = clip(**text_inputs).last_hidden_state

            output_handle = self.put_tensor(embeddings, device="cuda")

            peak_vram = 0
            if torch.cuda.is_available():
                torch.cuda.empty_cache()
                peak_vram = torch.cuda.max_memory_allocated() // (1024 * 1024)

            return ExecutionResult(
                success=True,
                outputs={"conditioning": output_handle},
                duration_us=0,
                peak_vram_mb=peak_vram,
            )

        except Exception as e:
            logger.exception(f"CLIP encode failed: {e}")
            return ExecutionResult(
                success=False,
                outputs={},
                duration_us=0,
                peak_vram_mb=0,
                error=str(e),
            )
