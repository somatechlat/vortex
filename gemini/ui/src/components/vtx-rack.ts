import { LitElement, html, css } from 'lit';
import { customElement, property, state } from 'lit/decorators.js';
import { repeat } from 'lit/directives/repeat.js';
import { virtualize } from '@lit-labs/virtualizer/virtualize.js';
import { RackController, RackUnit } from '../controllers/rack-controller.js';
import './vtx-blade.js';

@customElement('vtx-rack')
export class VortexRack extends LitElement {
    static styles = css`
    :host {
      display: block;
      height: 100%;
      width: 100%;
      overflow: hidden;
    }

    .container {
      height: 100%;
      width: 100%;
      padding: var(--vtx-space-lg);
      overflow-y: auto;
    }

    .scroller {
      display: flex;
      flex-direction: column;
      align-items: center;
      max-width: 800px;
      margin: 0 auto;
    }
  `;

    private rack = new RackController(this);

    render() {
        return html`
      <div class="container">
        <div class="scroller">
          ${virtualize({
            items: this.rack.units,
            renderItem: (unit: RackUnit) => html`
              <vtx-blade
                .id=${unit.id}
                .type=${unit.type}
                .status=${unit.status}
                .progress=${unit.progress}
                .selected=${this.rack.selectedId === unit.id}
                @click=${() => this.rack.select(unit.id)}
                @delete-unit=${(e: CustomEvent) => this.rack.removeUnit(e.detail.id)}
              ></vtx-blade>
            `,
        })}
        </div>
      </div>
    `;
    }
}
