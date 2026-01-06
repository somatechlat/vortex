<script lang="ts">
  import '../app.css';
  
  interface Props {
    children: import('svelte').Snippet;
  }
  
  const { children }: Props = $props();
</script>

<div class="app">
  {@render children()}
</div>

<style>
  .app {
    width: 100vw;
    height: 100vh;
    overflow: hidden;
    position: relative;
  }
</style>
