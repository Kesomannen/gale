import { listen } from '@tauri-apps/api/event';

import * as api from '$lib/api';
import type { DedicatedServerStatus } from '$lib/types';

class ServerState {
	status: DedicatedServerStatus = $state({
		state: 'stopped'
	});

	constructor() {
		void this.refresh();

		void listen<DedicatedServerStatus>('server_status_changed', (event) => {
			this.status = event.payload;
		});
	}

	async refresh() {
		this.status = await api.profile.server.getStatus();
	}

	isProfileLocked(profileId: number): boolean {
		return this.status.state === 'running' && this.status.profileId === profileId;
	}
}

const server = new ServerState();

export default server;
