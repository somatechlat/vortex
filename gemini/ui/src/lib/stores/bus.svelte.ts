/**
 * VORTEX Signal Bus Store - Svelte 5 Runes
 * 
 * Manages the 8 signal bus lanes for data flow routing.
 * Per SDD §3.1.2 Bus Store specification.
 * 
 * @author The Sage (10-Persona Collective)
 */

import { rackStore } from './rack.svelte';

export type DataType = 'LATENT' | 'IMAGE' | 'MODEL' | 'CLIP' | 'VAE' | 'CONDITIONING' | 'MASK' | 'CONTROLNET';

export interface BusLane {
    id: number;           // 0-7
    label: string;        // "Juggernaut XL Latent"
    type: DataType;
    sourceUnit: string;   // UnitID writing to this lane
    sourcePort: string;   // Port name
}

class BusStore {
    lanes = $state<BusLane[]>(
        Array.from({ length: 8 }, (_, i) => ({
            id: i,
            label: '',
            type: 'LATENT' as DataType,
            sourceUnit: '',
            sourcePort: '',
        }))
    );

    // Derived: Active lanes (have a source)
    activeLanes = $derived(
        this.lanes.filter(l => l.sourceUnit !== '')
    );

    // Derived: Available lanes (no source)
    availableLanes = $derived(
        this.lanes.filter(l => l.sourceUnit === '')
    );

    /**
     * Connect a unit output to a lane
     * @param laneId - Lane ID (0-7)
     * @param unitId - Source unit ID
     * @param portName - Source port name
     * @param type - Data type
     * @param label - Human-readable label
     */
    connect(laneId: number, unitId: string, portName: string, type: DataType, label: string): void {
        if (laneId < 0 || laneId > 7) return;

        const lane = this.lanes[laneId];
        lane.sourceUnit = unitId;
        lane.sourcePort = portName;
        lane.type = type;
        lane.label = label;
    }

    /**
     * Disconnect a lane
     * @param laneId - Lane ID to disconnect
     */
    disconnect(laneId: number): void {
        if (laneId < 0 || laneId > 7) return;

        const lane = this.lanes[laneId];
        lane.sourceUnit = '';
        lane.sourcePort = '';
        lane.label = '';
    }

    /**
     * Get available lanes for a unit to tap
     * A lane is available if it has a source AND the source comes
     * before the unit in the rack order.
     */
    getAvailableForUnit(unitId: string): BusLane[] {
        const unit = rackStore.units.find(u => u.id === unitId);
        if (!unit) return [];

        const unitIndex = unit.index;

        return this.lanes.filter(lane => {
            if (lane.sourceUnit === '') return false;

            // Get source unit index
            const sourceUnit = rackStore.units.find(u => u.id === lane.sourceUnit);
            return sourceUnit && sourceUnit.index < unitIndex;
        });
    }

    /**
     * Clear all connections from a unit
     * Called when a unit is removed
     */
    clearUnit(unitId: string): void {
        this.lanes.forEach(lane => {
            if (lane.sourceUnit === unitId) {
                lane.sourceUnit = '';
                lane.sourcePort = '';
                lane.label = '';
            }
        });
    }
}

export const busStore = new BusStore();
