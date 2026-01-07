<!-- 
  VORTEX Signal Bus Component
  Vertical multi-lane data highway
  Per SDD §3.2.3 and SCR-MAIN specification
  
  @author The Architect (10-Persona Collective)
-->
<script lang="ts">
  import { busStore, type DataType } from '$lib/stores/bus.svelte';
  import { execStore } from '$lib/stores/execution.svelte';
  
  // Lane color mapping
  const laneColors: Record<DataType, string> = {
    LATENT: 'var(--vtx-bus-1)',
    IMAGE: 'var(--vtx-bus-2)',
    MODEL: 'var(--vtx-bus-3)',
    CLIP: 'var(--vtx-bus-4)',
    VAE: 'var(--vtx-bus-5)',
    CONDITIONING: 'var(--vtx-bus-6)',
    MASK: 'var(--vtx-bus-7)',
    CONTROLNET: 'var(--vtx-bus-8)',
  };
  
  let hoveredLane = $state<number | null>(null);
</script>

<aside class="bus" role="region" aria-label="Signal Bus">
  <div class="bus__lanes">
    {#each busStore.lanes as lane (lane.id)}
      <div
        class="bus__lane"
        class:bus__lane--active={lane.sourceUnit !== ''}
        class:bus__lane--hover={hoveredLane === lane.id}
        class:bus__lane--executing={execStore.isExecuting && lane.sourceUnit !== ''}
        style:--lane-color={laneColors[lane.type]}
        onmouseenter={() => hoveredLane = lane.id}
        onmouseleave={() => hoveredLane = null}
        role="status"
        aria-label="Lane {lane.id + 1}: {lane.label || 'Empty'}"
      >
        <span class="bus__lane-line"></span>
      </div>
    {/each}
  </div>
  
  <!-- Tooltip -->
  {#if hoveredLane !== null}
    {@const lane = busStore.lanes[hoveredLane]}
    {#if lane.sourceUnit}
      <div class="bus__tooltip">
        <span class="bus__tooltip-type">{lane.type}</span>
        <span class="bus__tooltip-label">{lane.label}</span>
      </div>
    {/if}
  {/if}
</aside>

<style>
  .bus {
    position: relative;
    height: 100%;
    padding: var(--vtx-space-lg) var(--vtx-space-sm);
  }
  
  .bus__lanes {
    display: flex;
    gap: 6px;
    height: 100%;
  }
  
  .bus__lane {
    width: 8px;
    height: 100%;
    position: relative;
  }
  
  .bus__lane-line {
    position: absolute;
    inset: 0;
    background: var(--lane-color, var(--vtx-glass-border));
    opacity: 0.2;
    border-radius: 4px;
    transition: opacity var(--vtx-transition-fast);
  }
  
  .bus__lane--active .bus__lane-line {
    opacity: 0.6;
    box-shadow: 0 0 12px var(--lane-color);
  }
  
  .bus__lane--hover .bus__lane-line {
    opacity: 1;
  }
  
  .bus__lane--executing .bus__lane-line {
    animation: pulse 1s ease-in-out infinite;
  }
  
  @keyframes pulse {
    0%, 100% { opacity: 0.4; }
    50% { opacity: 1; }
  }
  
  .bus__tooltip {
    position: absolute;
    left: 100%;
    top: 50%;
    transform: translateY(-50%);
    margin-left: var(--vtx-space-sm);
    background: var(--vtx-glass);
    backdrop-filter: blur(var(--vtx-blur));
    border: 1px solid var(--vtx-glass-border);
    border-radius: var(--vtx-radius-sm);
    padding: var(--vtx-space-xs) var(--vtx-space-sm);
    white-space: nowrap;
    z-index: var(--vtx-z-overlay);
  }
  
  .bus__tooltip-type {
    font-family: var(--vtx-font-mono);
    font-size: var(--vtx-text-micro);
    text-transform: uppercase;
    opacity: 0.6;
    display: block;
  }
  
  .bus__tooltip-label {
    font-size: var(--vtx-text-micro);
  }
</style>
