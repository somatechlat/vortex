/**
 * VORTEX Rack Store - Svelte 5 Runes
 * 
 * Manages the vertical stack of processing units (Rack).
 * Per SDD §3.1.1 Rack Store specification.
 * 
 * @author The Craftsman (10-Persona Collective)
 */

import { SvelteMap } from 'svelte/reactivity';

export interface RackUnit {
    id: string;
    type: string;              // "com.vortex.loader", etc.
    index: number;             // Vertical position
    input_taps: Map<string, number>;   // PortName -> LaneID
    output_lanes: Map<string, number>; // PortName -> LaneID
    params: Record<string, unknown>;
    $status: 'IDLE' | 'RUNNING' | 'ERROR';
    $progress: number;
}

class RackStore {
    units = $state<RackUnit[]>([]);
    selectedId = $state<string | null>(null);

    // Derived: Units in render order
    sortedUnits = $derived(
        [...this.units].sort((a, b) => a.index - b.index)
    );

    // Derived: Selected unit
    selectedUnit = $derived(
        this.units.find(u => u.id === this.selectedId) ?? null
    );

    // Derived: Unit count
    count = $derived(this.units.length);

    /**
     * Add a new unit to the rack
     * @param type - Unit type identifier (e.g., "com.vortex.loader")
     * @returns The new unit ID
     */
    addUnit(type: string): string {
        const id = crypto.randomUUID();
        const index = this.units.length > 0
            ? Math.max(...this.units.map(u => u.index)) + 1
            : 0;

        this.units.push({
            id,
            type,
            index,
            input_taps: new SvelteMap(),
            output_lanes: new SvelteMap(),
            params: {},
            $status: 'IDLE',
            $progress: 0,
        });

        return id;
    }

    /**
     * Remove a unit from the rack
     * @param id - Unit ID to remove
     */
    removeUnit(id: string): void {
        const idx = this.units.findIndex(u => u.id === id);
        if (idx === -1) return;

        const removedIndex = this.units[idx].index;
        this.units.splice(idx, 1);

        // Reindex remaining units
        this.units.forEach(u => {
            if (u.index > removedIndex) u.index--;
        });

        // Clear selection if removed
        if (this.selectedId === id) {
            this.selectedId = null;
        }
    }

    /**
     * Move a unit to a new position
     * @param id - Unit ID to move
     * @param newIndex - New position index
     */
    moveUnit(id: string, newIndex: number): void {
        const unit = this.units.find(u => u.id === id);
        if (!unit) return;

        const oldIndex = unit.index;
        if (oldIndex === newIndex) return;

        // Shift other units
        this.units.forEach(u => {
            if (u.id === id) {
                u.index = newIndex;
            } else if (oldIndex < newIndex) {
                // Moving down: shift units in between up
                if (u.index > oldIndex && u.index <= newIndex) {
                    u.index--;
                }
            } else {
                // Moving up: shift units in between down
                if (u.index >= newIndex && u.index < oldIndex) {
                    u.index++;
                }
            }
        });
    }

    /**
     * Update unit parameters
     * @param id - Unit ID
     * @param params - New parameters to merge
     */
    updateParams(id: string, params: Record<string, unknown>): void {
        const unit = this.units.find(u => u.id === id);
        if (unit) {
            unit.params = { ...unit.params, ...params };
        }
    }

    /**
     * Set unit execution status
     * @param id - Unit ID
     * @param status - New status
     * @param progress - Optional progress (0-1)
     */
    setStatus(id: string, status: RackUnit['$status'], progress?: number): void {
        const unit = this.units.find(u => u.id === id);
        if (unit) {
            unit.$status = status;
            if (progress !== undefined) {
                unit.$progress = Math.max(0, Math.min(1, progress));
            }
        }
    }

    /**
     * Select a unit
     * @param id - Unit ID to select (null to deselect)
     */
    select(id: string | null): void {
        this.selectedId = id;
    }
}

export const rackStore = new RackStore();
