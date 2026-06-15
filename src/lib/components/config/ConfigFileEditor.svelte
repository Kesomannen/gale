<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import config from '$lib/state/config.svelte';
	import type { ConfigFileData, ConfigSection } from '$lib/types';
	import SmallHeading from '../prefs/SmallHeading.svelte';
	import InfoBox from '../ui/InfoBox.svelte';
	import ResetButton from '../ui/ResetButton.svelte';
	import SearchBar from '../ui/SearchBar.svelte';
	import ConfigEntryField from './ConfigEntryField.svelte';
	import { confirm } from '@tauri-apps/plugin-dialog';

	type Props = {
		file: ConfigFileData;
		locked: boolean;
	};

	let { file, locked }: Props = $props();

	const LARGE_FILE_ENTRY_COUNT = 100;

	let search = $state('');

	$effect(() => {
		file;
		search = '';
	});

	async function resetAll() {
		const confirmed = await confirm(m.config_resetAllConfirm_message({ name: file.displayName }), {
			title: m.config_resetAllConfirm_title()
		});

		if (!confirmed) return;
		await config.resetFile(file);
	}

	function sumEntryCount(sections: ConfigSection[]) {
		return sections.reduce((acc, section) => acc + section.entries.length, 0);
	}

	function onSectionClick(section: ConfigSection) {
		search = section.name;
	}

	let filteredSections = $derived.by(() => {
		const lowerSearch = search.toLowerCase().trim();

		return file.sections
			.map((section) => {
				if (section.name.toLowerCase().includes(lowerSearch)) {
					return section;
				}

				return {
					...section,
					entries: section.entries.filter((entry) => {
						return entry.name.toLowerCase().includes(lowerSearch);
					})
				};
			})
			.filter((section) => section.entries.length > 0);
	});

	let filteredEntryCount = $derived(sumEntryCount(filteredSections));
	let overflowing = $derived(filteredEntryCount > LARGE_FILE_ENTRY_COUNT);

	let totalEntryCount = $derived(sumEntryCount(file.sections));
	let isLarge = $derived(totalEntryCount > LARGE_FILE_ENTRY_COUNT);
</script>

{#if file.metadata}
	<div class="text-primary-400 mb-1 font-medium">
		{m.configFileEditor_metadata({
			name: file.metadata.modName,
			version: file.metadata.modVersion
		})}
	</div>
{/if}

<div class="flex gap-2">
	<div class="relative grow">
		<SearchBar bind:value={search} placeholder={m.configFileEditor_searchPlaceholder()} />
	</div>

	<ResetButton onclick={resetAll} label={m.configFileEditor_resetAll()} />
</div>

{#if isLarge}
	<SmallHeading>{m.configFileEditor_sections()}</SmallHeading>

	<div class="text-left">
		{#each filteredSections as section (section)}
			<button
				class="text-accent-400 hover:text-accent-300 block hover:underline"
				onclick={() => onSectionClick(section)}
			>
				{section.name}
			</button>
		{/each}
	</div>
{/if}

{#if overflowing}
	<InfoBox type="warning">
		{m.configFileEditor_largeFileWarning({
			count: filteredEntryCount,
			threshold: LARGE_FILE_ENTRY_COUNT
		})}
	</InfoBox>
{/if}

{#if !overflowing}
	{#each filteredSections as section (section)}
		<SmallHeading class="mb-1">
			{section.name}
		</SmallHeading>

		{#each section.entries as entry (entry)}
			<ConfigEntryField
				{locked}
				entryId={{
					file,
					section,
					entry
				}}
			/>
		{/each}
	{/each}
{/if}
