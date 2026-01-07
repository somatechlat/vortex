<!-- 
  VORTEX Cinematic Background Component
  WebGL2 animated gradient with bloom effect
  Per SDD §3.3 WebGL Renderer specification
  
  @author The Diplomat (10-Persona Collective - Premium Aesthetics)
-->
<script lang="ts">
  import { onMount } from 'svelte';
  
  interface Props {
    progress?: number;  // 0-1 execution progress
  }
  
  const { progress = 0 }: Props = $props();
  
  let canvas: HTMLCanvasElement;
  let animationId: number;
  
  onMount(() => {
    const gl = canvas.getContext('webgl2');
    if (!gl) {
      console.warn('WebGL2 not supported, falling back to CSS');
      return;
    }
    
    // Resize handler
    function resize() {
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;
      gl.viewport(0, 0, canvas.width, canvas.height);
    }
    resize();
    window.addEventListener('resize', resize);
    
    // Vertex shader - fullscreen quad
    const vertexShader = gl.createShader(gl.VERTEX_SHADER)!;
    gl.shaderSource(vertexShader, `#version 300 es
      in vec2 a_position;
      out vec2 v_uv;
      void main() {
        v_uv = a_position * 0.5 + 0.5;
        gl_Position = vec4(a_position, 0.0, 1.0);
      }
    `);
    gl.compileShader(vertexShader);
    
    // Fragment shader - cinematic gradient with bloom
    const fragmentShader = gl.createShader(gl.FRAGMENT_SHADER)!;
    gl.shaderSource(fragmentShader, `#version 300 es
      precision highp float;
      
      in vec2 v_uv;
      out vec4 fragColor;
      
      uniform float u_time;
      uniform float u_progress;
      uniform vec2 u_resolution;
      
      // Colors from Mitchell Hybrid palette
      const vec3 VOID = vec3(0.039, 0.039, 0.039);
      const vec3 TEAL = vec3(0.059, 0.298, 0.361);
      const vec3 MINT = vec3(0.596, 0.867, 0.792);
      
      void main() {
        vec2 uv = v_uv;
        vec2 center = vec2(0.5);
        
        // Distance from center
        float d = length(uv - center);
        
        // Breathing animation
        float breath = sin(u_time * 0.5) * 0.5 + 0.5;
        
        // Bloom intensity based on progress
        float bloomIntensity = 0.15 + u_progress * 0.3 + breath * 0.1;
        
        // Radial gradient
        float gradient = 1.0 - smoothstep(0.0, 0.8, d);
        
        // Mix colors
        vec3 color = mix(VOID, TEAL, gradient * bloomIntensity);
        
        // Add subtle bloom highlight
        float highlight = exp(-d * d * 8.0) * bloomIntensity * 0.5;
        color += MINT * highlight;
        
        // Vignette
        float vignette = 1.0 - d * 0.3;
        color *= vignette;
        
        fragColor = vec4(color, 1.0);
      }
    `);
    gl.compileShader(fragmentShader);
    
    // Program
    const program = gl.createProgram()!;
    gl.attachShader(program, vertexShader);
    gl.attachShader(program, fragmentShader);
    gl.linkProgram(program);
    gl.useProgram(program);
    
    // Fullscreen quad
    const positions = new Float32Array([
      -1, -1,  1, -1,  -1, 1,
      -1,  1,  1, -1,   1, 1,
    ]);
    const positionBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, positions, gl.STATIC_DRAW);
    
    const positionLocation = gl.getAttribLocation(program, 'a_position');
    gl.enableVertexAttribArray(positionLocation);
    gl.vertexAttribPointer(positionLocation, 2, gl.FLOAT, false, 0, 0);
    
    // Uniforms
    const timeLocation = gl.getUniformLocation(program, 'u_time');
    const progressLocation = gl.getUniformLocation(program, 'u_progress');
    const resolutionLocation = gl.getUniformLocation(program, 'u_resolution');
    
    // Render loop
    let startTime = performance.now();
    
    function render() {
      const time = (performance.now() - startTime) / 1000;
      
      gl.uniform1f(timeLocation, time);
      gl.uniform1f(progressLocation, progress);
      gl.uniform2f(resolutionLocation, canvas.width, canvas.height);
      
      gl.drawArrays(gl.TRIANGLES, 0, 6);
      
      animationId = requestAnimationFrame(render);
    }
    
    render();
    
    return () => {
      cancelAnimationFrame(animationId);
      window.removeEventListener('resize', resize);
    };
  });
</script>

<canvas 
  bind:this={canvas}
  class="cinematic-bg"
  aria-hidden="true"
></canvas>

<style>
  .cinematic-bg {
    position: fixed;
    inset: 0;
    z-index: var(--vtx-z-canvas);
    pointer-events: none;
  }
</style>
