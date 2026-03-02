import { html, render } from 'lit';
import './src/index.css';
import './src/components/vtx-rack.js';

const app = document.querySelector('#app');

if (app) {
    render(
        html`
      <div class="vortex-app">
        <vtx-rack></vtx-rack>
        <!-- Cinematic Background Layer -->
        <div class="vortex-bloom" aria-hidden="true"></div>
      </div>
    `,
        app
    );
}
