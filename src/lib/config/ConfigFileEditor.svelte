<script lang="ts">
	import type { ConfigEntry, ConfigFileData, ConfigSection } from '$lib/models';
	import ConfigEntryField from './ConfigEntryField.svelte';

	export let file: ConfigFileData;
	export let section: ConfigSection | null;

	export let locked: boolean;

	let search = '';

	$: ({ metadata } = file);

	$: shownEntries = section === null ? [] : filterEntries(section.entries, search);

	function filterEntries(entries: ConfigEntry[], search: string) {
		return entries.filter((entry) => {
			return entry.name.toLowerCase().includes(search.toLowerCase());
		});
	}
</script>

{#if metadata}
	<div class="text-primary-400 mb-1 px-4 font-medium">
		Created by {metadata.modName}
		{metadata.modVersion}
	</div>
{/if}

{#if section !== null}
	{#each shownEntries as entry (entry)}
		<ConfigEntryField
			{locked}
			entryId={{
				file,
				section,
				entry
			}}
		/>
	{/each}
{/if}
