<script lang="ts">
	import * as api from '$lib/api';
	import { capitalize } from '$lib/util';
	import ExpandedConfigEntryDialog from '$lib/components/dialogs/ExpandedConfigEntryDialog.svelte';

	import Button from '$lib/components/ui/Button.svelte';
	import ConfigFileEditor from '$lib/components/config/ConfigFileEditor.svelte';
	import ProfileLockedBanner from '$lib/components/mod-list/ProfileLockedBanner.svelte';
	import ConfigFileList from '$lib/components/config/ConfigFileList.svelte';
	import profiles from '$lib/state/profile.svelte';
	import { m } from '$lib/paraglide/messages';
	import LargeHeading from '$lib/components/prefs/LargeHeading.svelte';
	import config from '$lib/state/config.svelte';
	import { untrack } from 'svelte';
	import HelpCard from '$lib/components/ui/HelpCard.svelte';

	const selectedFile = $derived(config.selectedFile);

	$effect(() => {
		profiles.activeId;
		untrack(() => config.refresh());
	});
</script>

{#if !config.loading && config.files.length === 0}
	<HelpCard title={m.config_noFiles()} icon="ph:folder-open-fill" class="w-full" />
{:else}
	<div class="grid w-full grid-cols-[15rem_1fr] xl:grid-cols-[20rem_1fr]">
		{#if config.loading}
			{@render loadingSkeletons()}
		{:else}
			<ConfigFileList />
		{/if}

		<div class="overflow-y-auto px-6 pb-6">
			{#if profiles.activeLocked}
				<ProfileLockedBanner class="mt-4 mb-4" />
			{/if}

			{#if selectedFile}
				<LargeHeading class="mb-2 truncate">
					{selectedFile.relativePath}
				</LargeHeading>

				{#if selectedFile.type === 'ok'}
					<ConfigFileEditor file={selectedFile} locked={profiles.activeLocked} />
				{:else if selectedFile.type === 'unsupported'}
					<div class="text-primary-400 mb-1">
						{m.config_unsupported_content()}
					</div>
					<Button
						class="max-w-max"
						color="primary"
						onclick={() => api.config.openFile(selectedFile!)}
						icon="ph:arrow-square-out-fill"
					>
						{m.config_unsupported_button()}
					</Button>
				{:else if selectedFile.type === 'err'}
					<div class="text-primary-400 mb-1">
						{m.config_err_content()}
					</div>
					<code class="bg-primary-900 mb-1 flex rounded-sm p-4 text-red-500">
						{capitalize(selectedFile.error)}
					</code>
					<Button
						class="max-w-max"
						color="primary"
						onclick={() => api.config.openFile(selectedFile!)}
						icon="ph:arrow-square-out-fill"
					>
						{m.config_err_button()}
					</Button>
				{/if}
			{:else if !config.loading}
				<HelpCard title={m.config_content()} icon="ph:faders-fill" class="h-full" />
			{/if}
		</div>
	</div>
{/if}

{#snippet loadingSkeletons()}
	<div class="space-y-1 px-2 pt-10">
		{#each [0.9, 1, 0.8, 0.7] as width1, i (i)}
			{#each [0.8, 1, 0.7, 0.6, 0.9] as width2, j (j)}
				<div
					class="bg-primary-700 h-7 animate-pulse rounded"
					style="width: {width1 * width2 * 100}%;"
				></div>
			{/each}
		{/each}
	</div>
{/snippet}

<ExpandedConfigEntryDialog />
