import * as api from '$lib/api';
import type { BaseConfigFile, ConfigFile } from '$lib/types';

class ConfigState {
	files: ConfigFile[] = $state([]);
	selectedFile: ConfigFile | null = $state(null);
	loading = $state(false);

	async refresh() {
		if (this.loading) return;
		this.loading = true;
		try {
			console.log('Refreshing config files');
			this.files = await api.config.getFiles();

			const selectedPath = this.selectedFile?.relativePath;
			if (selectedPath) {
				this.selectedFile = this.findFileByPath(selectedPath);
			}
		} finally {
			this.loading = false;
		}
	}

	async deleteFile(file: BaseConfigFile) {
		await api.config.deleteFile(file);
		if (this.selectedFile === file) this.selectedFile = null;
		await this.refresh();
	}

	async resetFile(file: BaseConfigFile) {
		await api.config.resetAll(file);
		await this.refresh();
	}

	findFileByPath(path: string): ConfigFile | null {
		return this.files.find((f) => f.relativePath === path) ?? null;
	}
}

const config = new ConfigState();

config.refresh();

export default config;
