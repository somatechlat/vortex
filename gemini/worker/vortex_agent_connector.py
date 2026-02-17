#!/usr/bin/env python3
"""
VORTEX Agent Connector
Connects SomaAgent01 to VORTEX API for job execution
"""

import json
import sys
import requests
from typing import Dict, Any, List

class VortexConnector:
    """Connector to submit agent workflows to VORTEX"""
    
    def __init__(self, api_url: str = "http://localhost:11188"):
        self.api_url = api_url
        self.session = requests.Session()
        
    def submit_graph(self, graph: Dict[str, Any]) -> Dict[str, Any]:
        """
        Submit a graph from SomaAgent01 to VORTEX
        
        Args:
            graph: GraphDSL structure from SomaAgent01
            
        Returns:
            Response with graph_id and status
        """
        response = self.session.post(
            f"{self.api_url}/api/graph",
            json={"graph": graph},
            headers={"Content-Type": "application/json"}
        )
        response.raise_for_status()
        return response.json()
    
    def execute_graph(self, graph_id: str, output_nodes: List[str] = None) -> Dict[str, Any]:
        """
        Execute a submitted graph
        
        Args:
            graph_id: The graph ID from submit_graph
            output_nodes: Optional list of output node IDs
            
        Returns:
            Response with run_id and estimated time
        """
        payload = {}
        if output_nodes:
            payload["output_nodes"] = output_nodes
            
        response = self.session.post(
            f"{self.api_url}/api/graph/{graph_id}/execute",
            json=payload,
            headers={"Content-Type": "application/json"}
        )
        response.raise_for_status()
        return response.json()
    
    def submit_direct_job(self, node_type: str, params: Dict[str, Any], inputs: List[Dict]) -> Dict[str, Any]:
        """
        Submit a single job directly (for simpler agent tasks)
        
        Args:
            node_type: Executor type (e.g., "Sampler::KSampler")
            params: Parameters for the executor
            inputs: List of input tensor references
            
        Returns:
            Job result with metrics
        """
        job_id = f"agent-job-{id(params)}"  # Simple unique ID
        
        payload = {
            "job_id": job_id,
            "node_type": node_type,
            "params_json": params,
            "inputs": inputs
        }
        
        response = self.session.post(
            f"{self.api_url}/api/job",
            json=payload,
            headers={"Content-Type": "application/json"}
        )
        response.raise_for_status()
        return response.json()
    
    def get_status(self, run_id: str) -> Dict[str, Any]:
        """Get run status"""
        response = self.session.get(f"{self.api_url}/api/run/{run_id}/status")
        response.raise_for_status()
        return response.json()
    
    def get_workers(self) -> Dict[str, Any]:
        """Get list of workers"""
        response = self.session.get(f"{self.api_url}/api/workers")
        response.raise_for_status()
        return response.json()
    
    def spawn_worker(self, slot_id: int = 0) -> Dict[str, Any]:
        """Spawn a new worker"""
        response = self.session.post(
            f"{self.api_url}/api/workers",
            json={"action": "spawn", "slot_id": slot_id}
        )
        response.raise_for_status()
        return response.json()
    
    def get_system_info(self) -> Dict[str, Any]:
        """Get system information"""
        response = self.session.get(f"{self.api_url}/info")
        response.raise_for_status()
        return response.json()
    
    def get_metrics(self) -> str:
        """Get Prometheus metrics"""
        response = self.session.get(f"{self.api_url}/metrics")
        response.raise_for_status()
        return response.text


# Example usage
def example_soma_agent_workflow():
    """
    Example of how SomaAgent01 would generate a workflow and submit to VORTEX
    """
    connector = VortexConnector()
    
    print("="*70)
    print("VORTEX ↔ SomaAgent01 Integration Example")
    print("="*70)
    
    # Step 1: Check system status
    print("\n1. Checking VORTEX system...")
    try:
        info = connector.get_system_info()
        print(f"   ✅ VORTEX v{info['version']} running")
        print(f"   ✅ Uptime: {info['uptime_seconds']}s")
        print(f"   ✅ Workers: {info['workers']}")
    except Exception as e:
        print(f"   ❌ VORTEX not available: {e}")
        print("\n   Starting VORTEX first: cargo run --bin vortex-core")
        return
    
    # Step 2: Spawn worker if needed
    if info['workers'] == 0:
        print("\n2. Spawning worker...")
        connector.spawn_worker(0)
        print("   ✅ Worker spawned")
        import time
        time.sleep(1)
    
    # Step 3: Simple job submission (direct to worker)
    print("\n3. Submitting direct job...")
    job_result = connector.submit_direct_job(
        node_type="Loader::Checkpoint",
        params={"model_id": "runwayml/stable-diffusion-v1-5"},
        inputs=[]
    )
    print(f"   Job ID: {job_result['job_id']}")
    print(f"   Success: {job_result['success']}")
    if job_result.get('metrics'):
        m = job_result['metrics']
        print(f"   Execution: {m['execution_us']}μs")
        print(f"   Peak VRAM: {m['peak_vram_bytes']} bytes")
    
    # Step 4: Full graph submission (from SomaAgent01)
    print("\n4. Submitting full graph workflow...")
    
    # This would come from SomaAgent01's graph generator
    example_graph = {
        "nodes": [
            {"id": "load", "type": "Loader::Checkpoint", "params": {"model_id": "stabilityai/sdxl"}},
            {"id": "encode", "type": "Encoder::CLIP", "params": {"text": "a beautiful landscape"}},
            {"id": "sample", "type": "Sampler::KSampler", "params": {"steps": 20}},
            {"id": "decode", "type": "Decoder::VAE", "params": {}}
        ],
        "edges": [
            {"from": "load", "to": "sample"},
            {"from": "encode", "to": "sample"},
            {"from": "sample", "to": "decode"}
        ]
    }
    
    try:
        graph_response = connector.submit_graph(example_graph)
        print(f"   ✅ Graph submitted: {graph_response['graph_id']}")
        
        # Execute the graph
        execute_response = connector.execute_graph(graph_response['graph_id'])
        print(f"   ✅ Execution started: {execute_response['run_id']}")
        print(f"   ⏱️  Estimated time: {execute_response['estimated_time_ms']}ms")
        
    except Exception as e:
        print(f"   Note: Full graph execution requires database: {e}")
        print("   (Direct job submission works without DB)")
    
    # Step 5: Get metrics
    print("\n5. System metrics...")
    metrics = connector.get_metrics()
    print(metrics[:500] + "..." if len(metrics) > 500 else metrics)
    
    print("\n" + "="*70)
    print("✅ Integration test complete!")
    print("="*70)


if __name__ == "__main__":
    # Run example
    example_soma_agent_workflow()
