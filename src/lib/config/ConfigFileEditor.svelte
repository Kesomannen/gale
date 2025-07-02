<script lang="ts">
	import type { ConfigEntry, ConfigFileData, ConfigSection } from '$lib/types';
	import ConfigEntryField from './ConfigEntryField.svelte';

	type Props = {
		file: ConfigFileData;
		section: ConfigSection | null;
		locked: boolean;
	};

	let { file, section, locked }: Props = $props();

	let search = $state('');

	function filterEntries(entries: ConfigEntry[], search: string) {
		return entries.filter((entry) => {
			return entry.name.toLowerCase().includes(search.toLowerCase());
		});
	}

	let shownEntries = $derived(section === null ? [] : filterEntries(section.entries, search));
</script>

{#if file.metadata}
	<div class="text-primary-400 mb-1 px-4 font-medium">
		Created by {file.metadata.modName}
		{file.metadata.modVersion}
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
