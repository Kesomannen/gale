import { goto } from '$app/navigation';
import * as api from '$lib/api';
import type { BaseConfigFile, ConfigFile, ConfigSection } from '$lib/types';
import { untrack } from 'svelte';
import profiles from './profile.svelte';

class ConfigState {
	files: ConfigFile[] = $state([]);
	selectedFile: ConfigFile | null = $state(null);
	selectedSection: ConfigSection | null = $state(null);
	loading = $state(false);

	constructor() {
		$effect.root(() => {
			$effect(() => {
				profiles.activeId;
				untrack(() => this.refresh());
			});
		});
	}

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

	gotoModConfig(relativePath: string) {
		const file = config.findFileByPath(relativePath);
		if (!file) {
			console.error('Config file not found for mod', relativePath);
			return;
		}

		config.selectedFile = file;
		config.selectedSection = null;
		goto('/config');
	}
}

const config = new ConfigState();

export default config;
