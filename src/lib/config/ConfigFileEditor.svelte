<script lang="ts">
	import { T } from '$i18n';
	import type { ConfigEntry, ConfigFile, ConfigSection } from '$lib/models';
	import ConfigEntryField from './ConfigEntryField.svelte';

	export let file: ConfigFile;
	export let section: ConfigSection | undefined;

	let search = '';

	$: ({ metadata } = file);

	$: shownEntries =
		section === undefined
			? []
			: filterEntries(
					section.entries.filter((entry) => entry.type !== 'orphaned'),
					search
				);

	function filterEntries(entries: ConfigEntry[], search: string) {
		return entries.filter((entry) => {
			return entry.name.toLowerCase().includes(search.toLowerCase());
		});
	}
</script>

{#if metadata}
	<div class="font-medium text-slate-400">
		{T("Config created by", 
		{
			"pluginName": metadata.pluginName, 
			"pluginVersion": metadata.pluginVersion
		})}
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
