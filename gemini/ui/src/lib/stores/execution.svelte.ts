/**
 * VORTEX Execution Store - Svelte 5 Runes
 * 
 * Tracks graph execution state.
 * Per SDD §3.1.3 Execution Store specification.
 * 
 * @author The Witness (10-Persona Collective)
 */

export interface ExecutionState {
    id: string | null;
    status: 'IDLE' | 'QUEUED' | 'RUNNING' | 'COMPLETED' | 'FAILED';
    progress: number;
    activeNodeId: string | null;
    startedAt: number | null;
    result: unknown | null;
    error: string | null;
}

class ExecutionStore {
    state = $state<ExecutionState>({
        id: null,
        status: 'IDLE',
        progress: 0,
        activeNodeId: null,
        startedAt: null,
        result: null,
        error: null,
    });

    // Derived: Is executing
    isExecuting = $derived(
        this.state.status === 'RUNNING' || this.state.status === 'QUEUED'
    );

    // Derived: Elapsed time in seconds
    elapsedSeconds = $derived(() => {
        if (!this.state.startedAt) return 0;
        return Math.floor((Date.now() - this.state.startedAt) / 1000);
    });

    /**
     * Start execution
     * @param executionId - Execution ID from server
     */
    start(executionId: string): void {
        this.state = {
            id: executionId,
            status: 'RUNNING',
            progress: 0,
            activeNodeId: null,
            startedAt: Date.now(),
            result: null,
            error: null,
        };
    }

    /**
     * Update progress
     * @param nodeId - Currently executing node
     * @param progress - Overall progress (0-1)
     */
    updateProgress(nodeId: string, progress: number): void {
        this.state.activeNodeId = nodeId;
        this.state.progress = Math.max(0, Math.min(1, progress));
    }

    /**
     * Complete execution successfully
     * @param result - Execution result
     */
    complete(result: unknown): void {
        this.state.status = 'COMPLETED';
        this.state.progress = 1;
        this.state.result = result;
        this.state.activeNodeId = null;
    }

    /**
     * Mark execution as failed
     * @param error - Error message
     */
    fail(error: string): void {
        this.state.status = 'FAILED';
        this.state.error = error;
        this.state.activeNodeId = null;
    }

    /**
     * Reset to idle state
     */
    reset(): void {
        this.state = {
            id: null,
            status: 'IDLE',
            progress: 0,
            activeNodeId: null,
            startedAt: null,
            result: null,
            error: null,
        };
    }
}

export const execStore = new ExecutionStore();
