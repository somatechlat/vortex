import { ReactiveController, ReactiveControllerHost } from 'lit';

export interface RackUnit {
    id: string;
    type: string;
    status: 'IDLE' | 'RUNNING' | 'ERROR';
    progress: number;
    params: Record<string, any>;
}

export class RackController implements ReactiveController {
    host: ReactiveControllerHost;
    units: RackUnit[] = [];
    selectedId: string | null = null;

    constructor(host: ReactiveControllerHost) {
        (this.host = host).addController(this);
    }

    hostConnected() {
        // Initial sync / WebSocket connect
        console.log('RackController: hostConnected');
    }

    addUnit(type: string) {
        const id = crypto.randomUUID();
        this.units = [...this.units, {
            id,
            type,
            status: 'IDLE',
            progress: 0,
            params: {}
        }];
        this.host.requestUpdate();
    }

    removeUnit(id: string) {
        this.units = this.units.filter(u => u.id !== id);
        if (this.selectedId === id) this.selectedId = null;
        this.host.requestUpdate();
    }

    select(id: string) {
        this.selectedId = id;
        this.host.requestUpdate();
    }
}
