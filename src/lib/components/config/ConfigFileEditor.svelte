<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import config from '$lib/state/config.svelte';
	import type { ConfigFileData, ConfigSection } from '$lib/types';
	import SmallHeading from '../prefs/SmallHeading.svelte';
	import HelpCard from '../ui/HelpCard.svelte';
	import ResetButton from '../ui/ResetButton.svelte';
	import SearchBar from '../ui/SearchBar.svelte';
	import ConfigEntryField from './ConfigEntryField.svelte';
	import { confirm } from '@tauri-apps/plugin-dialog';

	type Props = {
		file: ConfigFileData;
		section: ConfigSection | null;
		locked: boolean;
	};

	let { file, section, locked }: Props = $props();

	let search = $state('');

	$effect(() => {
		section;
		search = '';
	});

	const filteredEntries = $derived.by(() => {
		if (!search) return section?.entries;

		const lowerSearch = search.toLowerCase();
		return section?.entries.filter((entry) => entry.name.toLowerCase().includes(lowerSearch));
	});

	async function resetAll() {
		const confirmed = await confirm(m.config_resetAllConfirm_message({ name: file.displayName }), {
			title: m.config_resetAllConfirm_title()
		});

		if (!confirmed) return;
		await config.resetFile(file);
	}
</script>

{#if file.metadata}
	<div class="text-primary-400">
		{m.configFileEditor_metadata({
			name: file.metadata.modName,
			version: file.metadata.modVersion
		})}
	</div>
{/if}

{#if section}
	<div class="mt-2 mb-4 flex gap-2">
		<div class="relative grow">
			<SearchBar bind:value={search} placeholder={m.configFileEditor_searchPlaceholder()} />
		</div>

		<ResetButton onclick={resetAll} label={m.configFileEditor_resetAll()} />
	</div>

	{#each filteredEntries as entry (entry)}
		<ConfigEntryField
			{locked}
			entryId={{
				file,
				section,
				entry
			}}
		/>
	{:else}
		<HelpCard icon="mdi:magnify" title={m.configFileEditor_noResults()} />
	{/each}
{:else}
	<SmallHeading>{m.configFileEditor_sections()}</SmallHeading>

	{#each file.sections as section}
		<button
			class="text-accent-400 hover:text-accent-300 block hover:underline"
			onclick={() => (config.selectedSection = section)}
		>
			{section.name}
		</button>
	{/each}
{/if}
