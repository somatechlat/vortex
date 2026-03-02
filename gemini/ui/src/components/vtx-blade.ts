import { LitElement, html, css } from 'lit';
import { customElement, property } from 'lit/decorators.js';

@customElement('vtx-blade')
export class VortexBlade extends LitElement {
  static styles = css`
    :host {
      display: block;
      margin-bottom: var(--vtx-space-md);
      cursor: pointer;
    }

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
    }

    .blade:hover {
      border-color: var(--vtx-glass-active);
    }

    .blade.selected {
      border-color: var(--vtx-mint);
    }

    .tap {
      display: flex;
      align-items: center;
      padding: var(--vtx-space-md);
    }

    .tap-dot {
      width: 12px;
      height: 12px;
      border-radius: 50%;
      background: var(--vtx-bus-1);
      box-shadow: 0 0 8px var(--vtx-bus-1);
    }

    .content {
      flex: 1;
      display: flex;
      flex-direction: column;
      padding: var(--vtx-space-md);
    }

    header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: var(--vtx-space-sm);
    }

    h3 {
      font-family: var(--vtx-font-mono);
      font-size: 10px;
      text-transform: uppercase;
      letter-spacing: 0.1em;
      margin: 0;
    }

    .handle {
      opacity: 0.4;
      cursor: grab;
    }

    .body {
      flex: 1;
    }

    footer {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-top: var(--vtx-space-sm);
      font-size: 10px;
      opacity: 0.6;
    }

    .status {
      display: flex;
      gap: var(--vtx-space-xs);
    }

    .progress {
      color: var(--vtx-mint);
    }

    .delete {
      background: none;
      border: none;
      color: inherit;
      cursor: pointer;
      opacity: 0;
      transition: opacity var(--vtx-transition-fast);
    }

    .blade:hover .delete {
      opacity: 0.6;
    }

    .delete:hover {
      opacity: 1;
      color: #ff4444;
    }
  `;

  @property({ type: String }) id = '';
  @property({ type: String }) type = 'UNIT';
  @property({ type: String }) status = 'IDLE';
  @property({ type: Number }) progress = 0;
  @property({ type: Boolean }) selected = false;

  render() {
    return html`
      <div class="blade ${this.selected ? 'selected' : ''}">
        <div class="tap">
          <span class="tap-dot"></span>
        </div>
        <div class="content">
          <header>
            <h3>${this.type.split('.').pop()?.toUpperCase()}</h3>
            <span class="handle">⋮⋮</span>
          </header>
          <div class="body">
            <slot></slot>
          </div>
          <footer>
            <div class="status">
              ${this.status}
              ${this.status === 'RUNNING' ? html`<span class="progress">${Math.round(this.progress * 100)}%</span>` : ''}
            </div>
            <button class="delete" @click=${this._onDelete}>✕</button>
          </footer>
        </div>
      </div>
    `;
  }

  private _onDelete(e: Event) {
    e.stopPropagation();
    this.dispatchEvent(new CustomEvent('delete-unit', {
      detail: { id: this.id },
      bubbles: true,
      composed: true
    }));
  }
}
