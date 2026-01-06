<script lang="ts">
  import CinematicBackground from '$lib/components/canvas/CinematicBackground.svelte';
  import Rack from '$lib/components/Rack.svelte';
  import SignalBus from '$lib/components/SignalBus.svelte';
  import { rackStore } from '$lib/stores/rack.svelte';
  
  // Execution state
  let isExecuting = $state(false);
  let progress = $state(0);
</script>

<svelte:head>
  <title>VORTEX - Flow Stream</title>
</svelte:head>

<!-- Cinematic WebGL Background -->
<CinematicBackground {progress} />

<!-- Main Workspace -->
<main class="workspace">
  <!-- Signal Bus (Left) -->
  <aside class="bus-container">
    <SignalBus />
  </aside>
  
  <!-- Rack (Center) -->
  <section class="rack-container">
    <Rack />
  </section>
  
  <!-- Sidebar (Right) - Inspector & Kernel AI -->
  <aside class="sidebar">
    <div class="sidebar__panel">
      <h2 class="sidebar__title">SIGNAL INSPECTOR</h2>
      <p class="sidebar__empty">Select a unit to inspect</p>
    </div>
  </aside>
</main>

<!-- Toolbox (Bottom) -->
<footer class="toolbox">
  <button class="toolbox__btn" onclick={() => rackStore.addUnit('com.vortex.loader')}>
    + Add Unit
  </button>
  <button class="toolbox__btn toolbox__btn--primary" disabled={isExecuting}>
    {isExecuting ? 'Rendering...' : '▶ Render'}
  </button>
</footer>

<style>
  .workspace {
    display: grid;
    grid-template-columns: var(--vtx-bus-width) 1fr var(--vtx-sidebar-width);
    height: 100vh;
    position: relative;
    z-index: var(--vtx-z-rack);
  }
  
  .bus-container {
    position: relative;
  }
  
  .rack-container {
    overflow-y: auto;
    padding: var(--vtx-space-lg);
  }
  
  .sidebar {
    background: var(--vtx-glass);
    backdrop-filter: blur(var(--vtx-blur));
    border-left: 1px solid var(--vtx-glass-border);
    padding: var(--vtx-space-lg);
  }
  
  .sidebar__title {
    font-family: var(--vtx-font-mono);
    font-size: var(--vtx-text-micro);
    text-transform: uppercase;
    letter-spacing: 0.1em;
    opacity: 0.6;
    margin-bottom: var(--vtx-space-md);
  }
  
  .sidebar__empty {
    font-size: var(--vtx-text-micro);
    opacity: 0.4;
  }
  
  .toolbox {
    position: fixed;
    bottom: var(--vtx-space-lg);
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    gap: var(--vtx-space-sm);
    padding: var(--vtx-space-sm);
    background: var(--vtx-glass);
    backdrop-filter: blur(var(--vtx-blur));
    border: 1px solid var(--vtx-glass-border);
    border-radius: var(--vtx-radius-pill);
    z-index: var(--vtx-z-overlay);
  }
  
  .toolbox__btn {
    padding: var(--vtx-space-sm) var(--vtx-space-md);
    border-radius: var(--vtx-radius-pill);
    font-size: var(--vtx-text-micro);
    font-weight: 500;
    transition: background var(--vtx-transition-fast);
    
    &:hover {
      background: var(--vtx-glass-hover);
    }
  }
  
  .toolbox__btn--primary {
    background: var(--vtx-mint);
    color: var(--vtx-void);
    
    &:hover {
      opacity: 0.9;
    }
    
    &:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }
  }
</style>
