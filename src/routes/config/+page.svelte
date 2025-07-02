<script lang="ts">
	import * as api from '$lib/api';
	import type { ConfigSection, ConfigFile } from '$lib/types';
	import { capitalize } from '$lib/util';
	import ExpandedEntryPopup from '$lib/config/ExpandedEntryPopup.svelte';

	import { activeProfileLocked } from '$lib/stores.svelte';
	import Button from '$lib/components/Button.svelte';
	import ConfigFileEditor from '$lib/config/ConfigFileEditor.svelte';
	import ProfileLockedBanner from '$lib/modlist/ProfileLockedBanner.svelte';
	import ConfigFileList from '$lib/config/ConfigFileList.svelte';

	let selectedFile: ConfigFile | null = $state(null);
	let selectedSection: ConfigSection | null = $state(null);
</script>

<div class="flex grow overflow-hidden">
	<ConfigFileList bind:selectedFile bind:selectedSection />

	<div class="flex max-w-4xl grow flex-col overflow-y-auto py-4">
		{#if $activeProfileLocked}
			<ProfileLockedBanner class="mx-4 mb-4" />
		{/if}

		{#if selectedFile !== null}
			<div class="shrink-0 truncate px-4 text-2xl font-bold text-white">
				{selectedFile.relativePath}
				{#if selectedSection}
					<span class="text-primary-400">/</span>
					{selectedSection.name.length > 0 ? selectedSection.name : '<Nameless section>'}
				{/if}
			</div>

			{#if selectedFile.type === 'ok'}
				<ConfigFileEditor
					file={selectedFile}
					section={selectedSection}
					locked={$activeProfileLocked}
				/>
			{:else if selectedFile.type === 'unsupported'}
				<div class="text-primary-400 mb-1 px-4">
					This file is in an unsupported format. Please open it in an external program to make
					changes.
				</div>
				<Button
					class="mx-4 max-w-max"
					color="primary"
					onclick={() => api.config.openFile(selectedFile!)}
					icon="mdi:open-in-new"
				>
					Open in external program
				</Button>
			{:else if selectedFile.type === 'err'}
				<div class="text-primary-400 mb-1 px-4">
					An error occured while reading this config file:
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
					Open in external program
				</Button>
			{/if}
		{:else}
			<div class="text-primary-400 flex w-full grow items-center justify-center text-lg">
				Select a config file to start editing
			</div>
		{/if}
	</div>
</div>

<ExpandedEntryPopup />
