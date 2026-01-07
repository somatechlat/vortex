"""Executor Framework for VORTEX Worker.

Provides the AbstractExecutor base class and ExecutorRegistry for
managing compute operations (nodes like KSampler, VAEDecode, etc).
"""

from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import Any, Dict, List, Optional, Type
import time
import logging

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
    outputs: Dict[str, TensorHandle]
    duration_us: int
    peak_vram_mb: int
    error: Optional[str] = None


class AbstractExecutor(ABC):
    """Base class for all executor implementations.
    
    Each ComfyUI node type maps to an executor that handles
    the actual compute logic.
    """
    
    # Class-level metadata
    OP_TYPE: str = ""
    INPUT_TYPES: Dict[str, str] = {}
    OUTPUT_TYPES: Dict[str, str] = {}
    
    def __init__(self, shm_arena):
        self.shm = shm_arena
    
    @abstractmethod
    def execute(
        self,
        inputs: Dict[str, TensorHandle],
        params: Dict[str, Any],
    ) -> ExecutionResult:
        """Execute the operation.
        
        Args:
            inputs: Map of input name to tensor handle
            params: Node parameters from the graph
            
        Returns:
            ExecutionResult with output handles and metrics
        """
        pass
    
    def get_tensor(self, handle: TensorHandle):
        """Load a tensor from shared memory."""
        # Placeholder - will use DLPack in production
        return None
    
    def put_tensor(self, tensor, device: str = "cuda") -> TensorHandle:
        """Store a tensor in shared memory."""
        # Placeholder - will use DLPack in production
        return TensorHandle(offset=0, shape=tensor.shape, dtype=str(tensor.dtype))


class ExecutorRegistry:
    """Registry for executor classes.
    
    Maps operation types (e.g., "Sampler::KSampler") to their
    executor implementations.
    """
    
    _executors: Dict[str, Type[AbstractExecutor]] = {}
    
    @classmethod
    def register(cls, op_type: str):
        """Decorator to register an executor class."""
        def decorator(executor_cls: Type[AbstractExecutor]):
            cls._executors[op_type] = executor_cls
            executor_cls.OP_TYPE = op_type
            logger.info(f"Registered executor: {op_type}")
            return executor_cls
        return decorator
    
    @classmethod
    def get(cls, op_type: str) -> Optional[Type[AbstractExecutor]]:
        """Get executor class for an operation type."""
        return cls._executors.get(op_type)
    
    @classmethod
    def list(cls) -> List[str]:
        """List all registered operation types."""
        return list(cls._executors.keys())
    
    @classmethod
    def execute_node(
        cls,
        op_type: str,
        inputs: Dict[str, TensorHandle],
        params: Dict[str, Any],
        shm_arena,
    ) -> ExecutionResult:
        """Execute a node by operation type."""
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


# ═══════════════════════════════════════════════════════════════
#                    EXAMPLE EXECUTORS
# ═══════════════════════════════════════════════════════════════

@ExecutorRegistry.register("Loader::Checkpoint")
class CheckpointLoader(AbstractExecutor):
    """Load a model checkpoint."""
    
    INPUT_TYPES = {}
    OUTPUT_TYPES = {"model": "MODEL", "clip": "CLIP", "vae": "VAE"}
    
    def execute(self, inputs, params) -> ExecutionResult:
        model_path = params.get("ckpt_name", "")
        logger.info(f"Loading checkpoint: {model_path}")
        # Placeholder - actual implementation would load the model
        return ExecutionResult(
            success=True,
            outputs={
                "model": TensorHandle(0, (), "model"),
                "clip": TensorHandle(0, (), "clip"),
                "vae": TensorHandle(0, (), "vae"),
            },
            duration_us=0,
            peak_vram_mb=4096,
        )


@ExecutorRegistry.register("Sampler::KSampler")
class KSamplerExecutor(AbstractExecutor):
    """KSampler diffusion sampling."""
    
    INPUT_TYPES = {"model": "MODEL", "positive": "CONDITIONING", "negative": "CONDITIONING", "latent": "LATENT"}
    OUTPUT_TYPES = {"samples": "LATENT"}
    
    def execute(self, inputs, params) -> ExecutionResult:
        steps = params.get("steps", 20)
        cfg = params.get("cfg", 7.0)
        sampler = params.get("sampler_name", "euler")
        scheduler = params.get("scheduler", "normal")
        
        logger.info(f"KSampler: steps={steps}, cfg={cfg}, sampler={sampler}")
        
        # Placeholder - actual implementation would run diffusion
        return ExecutionResult(
            success=True,
            outputs={"samples": TensorHandle(0, (1, 4, 64, 64), "float16")},
            duration_us=0,
            peak_vram_mb=2048,
        )


@ExecutorRegistry.register("Decoder::VAE")
class VAEDecodeExecutor(AbstractExecutor):
    """VAE decode latents to image."""
    
    INPUT_TYPES = {"samples": "LATENT", "vae": "VAE"}
    OUTPUT_TYPES = {"image": "IMAGE"}
    
    def execute(self, inputs, params) -> ExecutionResult:
        logger.info("VAE decoding latents to image")
        
        # Placeholder
        return ExecutionResult(
            success=True,
            outputs={"image": TensorHandle(0, (1, 512, 512, 3), "uint8")},
            duration_us=0,
            peak_vram_mb=1024,
        )


@ExecutorRegistry.register("Encoder::CLIP")
class CLIPTextEncode(AbstractExecutor):
    """CLIP text encoding for conditioning."""
    
    INPUT_TYPES = {"clip": "CLIP", "text": "STRING"}
    OUTPUT_TYPES = {"conditioning": "CONDITIONING"}
    
    def execute(self, inputs, params) -> ExecutionResult:
        text = params.get("text", "")
        logger.info(f"CLIP encoding: {text[:50]}...")
        
        return ExecutionResult(
            success=True,
            outputs={"conditioning": TensorHandle(0, (1, 77, 768), "float16")},
            duration_us=0,
            peak_vram_mb=256,
        )
