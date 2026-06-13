<script lang="ts">
	import * as api from '$lib/api';
	import type { ConfigFile } from '$lib/types';
	import { capitalize } from '$lib/util';
	import ExpandedConfigEntryDialog from '$lib/components/dialogs/ExpandedConfigEntryDialog.svelte';

	import Button from '$lib/components/ui/Button.svelte';
	import ConfigFileEditor from '$lib/components/config/ConfigFileEditor.svelte';
	import ProfileLockedBanner from '$lib/components/mod-list/ProfileLockedBanner.svelte';
	import ConfigFileList from '$lib/components/config/ConfigFileList.svelte';
	import profiles from '$lib/state/profile.svelte';
	import { m } from '$lib/paraglide/messages';
	import LargeHeading from '$lib/components/prefs/LargeHeading.svelte';
	import { confirm } from '@tauri-apps/plugin-dialog';

	let selectedFile: ConfigFile | null = $state(null);

	async function resetAll() {
		if (!selectedFile) return;
		const confirmed = await confirm(
			m.config_resetAllConfirm_message({ name: selectedFile.displayName }),
			{
				title: m.config_resetAllConfirm_title()
			}
		);

		if (!confirmed) return;
		await api.config.resetAll(selectedFile);
	}
</script>

<div class="flex grow overflow-y-auto">
	<ConfigFileList bind:selectedFile />

	<div class="grow overflow-y-auto px-6 pb-6">
		{#if profiles.activeLocked}
			<ProfileLockedBanner class="mb-4" />
		{/if}

		{#if selectedFile !== null}
			<LargeHeading class="mb-2 truncate">
				{selectedFile.relativePath}
			</LargeHeading>

			{#if selectedFile.type === 'ok'}
				<ConfigFileEditor file={selectedFile} locked={profiles.activeLocked} {resetAll} />
			{:else if selectedFile.type === 'unsupported'}
				<div class="text-primary-400 mb-1 px-4">
					{m.config_unsupported_content()}
				</div>
				<Button
					class="mx-4 max-w-max"
					color="primary"
					onclick={() => api.config.openFile(selectedFile!)}
					icon="mdi:open-in-new"
				>
					{m.config_unsupported_button()}
				</Button>
			{:else if selectedFile.type === 'err'}
				<div class="text-primary-400 mb-1 px-4">
					{m.config_err_content()}
				</div>
				<code class="bg-primary-900 mx-4 mb-1 flex rounded-sm p-4 text-red-500">
					{capitalize(selectedFile.error)}
				</code>
				<Button
					class="mx-4 max-w-max"
					color="primary"
					onclick={() => api.config.openFile(selectedFile!)}
					icon="icon=mdi:open-in-new"
				>
					{m.config_err_button()}
				</Button>
			{/if}
		{:else}
			<div class="text-primary-400 flex h-full w-full grow items-center justify-center text-lg">
				{m.config_content()}
			</div>
		{/if}
	</div>
</div>

<ExpandedConfigEntryDialog />
