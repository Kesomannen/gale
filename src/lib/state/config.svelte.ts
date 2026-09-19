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
	profileId: number | null = $state(null);
	private generation = 0;

	constructor() {
		$effect.root(() => {
			$effect(() => {
				const activeId = profiles.activeId;
				untrack(() => {
					// clear state for the previous profile immediately so a
					// switch can't show or mutate the wrong profile's config
					this.files = [];
					this.selectedFile = null;
					this.selectedSection = null;
					this.profileId = activeId;
					this.refresh();
				});
			});
		});
	}

	async refresh() {
		const profileId = this.profileId;
		if (profileId === null) {
			this.files = [];
			return;
		}

		const generation = ++this.generation;
		this.loading = true;
		try {
			const files = await api.config.getFiles(profileId);
			// a newer refresh or a profile switch makes this response stale
			// (a switch to no profile returns before bumping the generation)
			if (generation !== this.generation || this.profileId !== profileId) return;

			this.files = files;

			const selectedPath = this.selectedFile?.relativePath;
			if (selectedPath) {
				this.selectedFile = this.findFileByPath(selectedPath);
			}
		} finally {
			if (generation === this.generation) this.loading = false;
		}
	}

	async deleteFile(file: BaseConfigFile) {
		if (this.profileId === null) return;
		await api.config.deleteFile(file, this.profileId);
		if (this.selectedFile === file) this.selectedFile = null;
		await this.refresh();
	}

	async resetFile(file: BaseConfigFile) {
		if (this.profileId === null) return;
		await api.config.resetAll(file, this.profileId);
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
