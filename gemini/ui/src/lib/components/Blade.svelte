<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { RackUnit } from '$lib/stores/rack.svelte';
  import { rackStore } from '$lib/stores/rack.svelte';
  import { busStore } from '$lib/stores/bus.svelte';

  interface Props {
    unit: RackUnit;
    draggable?: boolean;
    ondragstart?: () => void;
    ondragend?: () => void;
    children?: Snippet;
  }
  
  const { unit, draggable = false, ondragstart, ondragend, children }: Props = $props();
  
  let statusClass = $derived(
    unit.$status === 'RUNNING' ? 'blade--running' :
    unit.$status === 'ERROR' ? 'blade--error' : ''
  );
  
  let unitName = $derived(
    unit.type.split('.').pop()?.toUpperCase() ?? 'UNIT'
  );
  
  function handleClick() {
    rackStore.select(unit.id);
  }
  
  function handleDelete(e: Event) {
    e.stopPropagation();
    rackStore.removeUnit(unit.id);
    busStore.clearUnit(unit.id);
  }
</script>

<div
  class="blade {statusClass}"
  class:blade--selected={rackStore.selectedId === unit.id}
  {draggable}
  {ondragstart}
  {ondragend}
  onclick={handleClick}
  onkeydown={(e) => e.key === 'Enter' && handleClick()}
  role="button"
  tabindex="0"
  aria-label="{unitName} unit card"
  data-testid="blade-{unit.id}"
>
  <div class="blade__tap" aria-label="Signal connection">
    <span class="blade__tap-dot"></span>
  </div>
  
  <div class="blade__content">
    <header class="blade__header">
      <h3 class="blade__title">{unitName}</h3>
      <span class="blade__handle" aria-label="Drag handle">⋮⋮</span>
    </header>
    
    <div class="blade__body">
      {#if children}
        {@render children()}
      {:else}
        <p class="blade__hint">Configure unit</p>
      {/if}
    </div>
    
    <footer class="blade__footer">
      <span class="blade__status">
        {unit.$status}
        {#if unit.$status === 'RUNNING'}
          <span class="blade__progress">{Math.round(unit.$progress * 100)}%</span>
        {/if}
      </span>
      <button 
        class="blade__delete" 
        onclick={handleDelete}
        aria-label="Delete unit"
      >
        ✕
      </button>
    </footer>
  </div>
</div>

<style>
  .blade {
    display: flex;
    background: var(--vtx-glass);
    backdrop-filter: blur(var(--vtx-blur));
    border: 1px solid var(--vtx-glass-border);
    border-radius: var(--vtx-radius);
    box-shadow: var(--vtx-shadow);
    min-height: var(--vtx-blade-min-height);
    transition: transform var(--vtx-transition-fast), 
                border-color var(--vtx-transition-fast);
    cursor: pointer;
    
    &:hover {
      border-color: var(--vtx-glass-active);
    }
    
    &:focus {
      outline: 2px solid var(--vtx-mint);
      outline-offset: 2px;
    }
  }
  
  .blade--selected {
    border-color: var(--vtx-mint);
  }
  
  .blade--running {
    border-color: var(--vtx-info);
  }
  
  .blade--error {
    border-color: var(--vtx-error);
  }
  
  .blade__tap {
    display: flex;
    align-items: center;
    padding: var(--vtx-space-md);
  }
  
  .blade__tap-dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--vtx-bus-1);
    box-shadow: 0 0 8px var(--vtx-bus-1);
  }
  
  .blade__content {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: var(--vtx-space-md);
  }
  
  .blade__header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--vtx-space-sm);
  }
  
  .blade__title {
    font-family: var(--vtx-font-mono);
    font-size: var(--vtx-text-micro);
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }
  
  .blade__handle {
    opacity: 0.4;
    cursor: grab;
    
    &:active {
      cursor: grabbing;
    }
  }
  
  .blade__body {
    flex: 1;
  }
  
  .blade__hint {
    font-size: var(--vtx-text-micro);
    opacity: 0.4;
  }
  
  .blade__footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: var(--vtx-space-sm);
    font-size: var(--vtx-text-micro);
    opacity: 0.6;
  }
  
  .blade__status {
    display: flex;
    gap: var(--vtx-space-xs);
  }
  
  .blade__progress {
    color: var(--vtx-mint);
  }
  
  .blade__delete {
    opacity: 0;
    transition: opacity var(--vtx-transition-fast);
    
    .blade:hover & {
      opacity: 0.6;
    }
    
    &:hover {
      opacity: 1;
      color: var(--vtx-error);
    }
  }
</style>
