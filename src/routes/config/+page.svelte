<script lang="ts">
	import * as api from '$lib/api';
	import type { ConfigSection, ConfigFile } from '$lib/types';
	import { capitalize } from '$lib/util';
	import ExpandedConfigEntryDialog from '$lib/components/dialogs/ExpandedConfigEntryDialog.svelte';

	import Button from '$lib/components/ui/Button.svelte';
	import ConfigFileEditor from '$lib/components/config/ConfigFileEditor.svelte';
	import ProfileLockedBanner from '$lib/components/mod-list/ProfileLockedBanner.svelte';
	import ConfigFileList from '$lib/components/config/ConfigFileList.svelte';
	import profiles from '$lib/state/profile.svelte';
	import { m } from '$lib/paraglide/messages';

	let selectedFile: ConfigFile | null = $state(null);
	let selectedSection: ConfigSection | null = $state(null);
</script>

<div class="flex grow overflow-hidden">
	<ConfigFileList bind:selectedFile bind:selectedSection />

	<div class="flex max-w-4xl grow flex-col overflow-y-auto py-4">
		{#if profiles.activeLocked}
			<ProfileLockedBanner class="mx-4 mb-4" />
		{/if}

		{#if selectedFile !== null}
			<div class="shrink-0 truncate px-4 text-2xl font-bold text-white">
				{selectedFile.relativePath}
				{#if selectedSection}
					<span class="text-primary-400">/</span>
					{selectedSection.name.length > 0 ? selectedSection.name : m.config_nameLess()}
				{/if}
			</div>

			{#if selectedFile.type === 'ok'}
				<ConfigFileEditor
					file={selectedFile}
					section={selectedSection}
					locked={profiles.activeLocked}
				/>
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
			<div class="text-primary-400 flex w-full grow items-center justify-center text-lg">
				{m.config_content()}
			</div>
		{/if}
	</div>
</div>

<ExpandedConfigEntryDialog />
