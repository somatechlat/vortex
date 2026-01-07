<!-- 
  VORTEX Rack Component
  Vertical stack of processing unit Blades
  Per SDD §3.2.1 Rack Component specification
-->
<script lang="ts">
  import { flip } from 'svelte/animate';
  import { rackStore } from '$lib/stores/rack.svelte';
  import Blade from './Blade.svelte';
  
  let draggedId: string | null = $state(null);
  let dropIndex: number | null = $state(null);
  
  function handleDragStart(id: string) {
    draggedId = id;
  }
  
  function handleDragOver(e: DragEvent, index: number) {
    e.preventDefault();
    dropIndex = index;
  }
  
  function handleDrop(e: DragEvent, targetIndex: number) {
    e.preventDefault();
    if (draggedId) {
      rackStore.moveUnit(draggedId, targetIndex);
    }
    draggedId = null;
    dropIndex = null;
  }
  
  function handleDragEnd() {
    draggedId = null;
    dropIndex = null;
  }
</script>

<div 
  class="rack"
  role="list"
  aria-label="Rack Units"
>
  {#each rackStore.sortedUnits as unit (unit.id)}
    <div
      class="rack__slot"
      class:rack__slot--drop-target={dropIndex === unit.index}
      ondragover={(e) => handleDragOver(e, unit.index)}
      ondrop={(e) => handleDrop(e, unit.index)}
      animate:flip={{ duration: 250 }}
    >
      <Blade
        {unit}
        draggable={true}
        ondragstart={() => handleDragStart(unit.id)}
        ondragend={handleDragEnd}
      />
    </div>
  {/each}
  
  {#if rackStore.count === 0}
    <div class="rack__empty">
      <p>No units in rack</p>
      <p class="rack__hint">Click "+ Add Unit" to begin</p>
    </div>
  {/if}
</div>

<style>
  .rack {
    display: flex;
    flex-direction: column;
    gap: var(--vtx-space-md);
    padding: var(--vtx-space-lg);
    min-height: 100vh;
  }
  
  .rack__slot {
    transition: transform var(--vtx-transition-fast);
  }
  
  .rack__slot--drop-target {
    transform: translateY(8px);
  }
  
  .rack__empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 200px;
    border: 2px dashed var(--vtx-glass-border);
    border-radius: var(--vtx-radius);
    opacity: 0.5;
  }
  
  .rack__hint {
    font-size: var(--vtx-text-micro);
    margin-top: var(--vtx-space-sm);
  }
</style>
