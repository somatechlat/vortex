import sys
import os
from pathlib import Path

# Add worker directory to sys.path to allow importing vortex_worker
worker_path = Path(__file__).parent.parent
sys.path.insert(0, str(worker_path))
