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
				// Whenever the user switches active profiles, refresh.
				profiles.activeId;
				untrack(() => this.refresh());
			});
		});
	}

	async refresh() {
		if (this.loading) return;
		console.log('Refreshing config files...');
		this.loading = true;
		try {
			this.files = await api.config.getFiles();

			if (this.selectedFile) {
				this.selectedFile = this.findFileByPath(this.selectedFile.relativePath);
			}

			if (this.selectedFile && this.selectedSection) {
				this.selectedSection = this.findSectionByName(this.selectedFile, this.selectedSection.name);
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

	findSectionByName(file: ConfigFile, name: string): ConfigSection | null {
		if (file.type !== 'ok') return null;
		return file.sections.find((s) => s.name === name) ?? null;
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
