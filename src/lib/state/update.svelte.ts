import { check, type Update } from '@tauri-apps/plugin-updater';

class UpdateState {
	next: Update | null = $state(null);
	isChecking = $state(false);

	refresh = async () => {
		if (this.isChecking) return;

		this.isChecking = true;
		this.next = await check();
		this.isChecking = false;
	};
}

const updates = new UpdateState();
export default updates;
