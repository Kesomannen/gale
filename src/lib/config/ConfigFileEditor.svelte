<script lang="ts">
	import type { ConfigEntry, ConfigFileData, ConfigSection } from '$lib/models';
	import ConfigEntryField from './ConfigEntryField.svelte';

	export let file: ConfigFileData;
	export let section: ConfigSection | undefined;

	let search = '';

	$: ({ metadata } = file);

	$: shownEntries = section === undefined ? [] : filterEntries(section.entries, search);

	function filterEntries(entries: ConfigEntry[], search: string) {
		return entries.filter((entry) => {
			return entry.name.toLowerCase().includes(search.toLowerCase());
		});
	}
</script>

{#if metadata}
	<div class="mb-1 px-4 font-medium text-slate-400">
		Created by {metadata.modName}
		{metadata.modVersion}
	</div>
{/if}

{#if section}
	{#each shownEntries as entry (entry)}
		<ConfigEntryField
			entryId={{
				file,
				section,
				entry
			}}
		/>
	{/each}
{/if}
