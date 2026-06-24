<script lang="ts">
	import * as api from '$lib/api';
	import IconButton from '$lib/components/ui/IconButton.svelte';
	import { m } from '$lib/paraglide/messages';
	import config from '$lib/state/config.svelte';
	import type { ConfigFile } from '$lib/types';
	import Icon from '@iconify/svelte';
	import { confirm } from '@tauri-apps/plugin-dialog';
	import { Collapsible } from 'bits-ui';
	import DropdownArrow from '../ui/DropdownArrow.svelte';

	type Props = {
		file: ConfigFile;
		duplicate: boolean;
		locked: boolean;
	};

	let { file, duplicate, locked }: Props = $props();

	let open = $state(false);

	let shownSections = $derived(
		file.type === 'ok' ? file.sections.filter((section) => section.entries.length > 0) : []
	);

	let fileLabel = $derived.by(() => {
		if (!file.displayName) {
			return { name: file.relativePath, disambiguator: null };
		}

		if (duplicate) {
			const firstFolder = file.relativePath.split('/')[0];
			if (firstFolder !== file.displayName) {
				return { name: file.displayName, disambiguator: firstFolder };
			}
		}

		return { name: file.displayName, disambiguator: null };
	});

	async function deleteFile() {
		let confirmed = await confirm(
			m.configFileListItem_deleteFile_confirm({ name: file.displayName ?? file.relativePath })
		);
		if (!confirmed) return;

		await config.deleteFile(file);
	}

	async function openFile() {
		await api.config.openFile(file);
	}
</script>

{#if file.type !== 'ok' || shownSections.length > 0}
	<Collapsible.Root bind:open>
		<Collapsible.Trigger
			onclick={(evt) => {
				if (file.type === 'ok') return;

				config.selectedFile = file;
				config.selectedSection = null;
				evt.preventDefault();
			}}
			class={[
				file.type === 'ok' && 'text-primary-200',
				file.type === 'unsupported' && 'text-primary-400',
				file.type === 'err' && 'text-red-500',
				config.selectedFile === file ? 'bg-primary-700' : 'hover:bg-primary-700',
				'group flex w-full items-center overflow-hidden rounded px-2 py-1'
			]}
		>
			{#if file.type === 'ok'}
				<DropdownArrow {open} />
			{:else}
				<Icon
					icon={file.type === 'unsupported' ? 'mdi:question-mark-circle' : 'mdi:alert-circle'}
					class="shrink-0"
				/>
			{/if}

			<div class="mr-auto shrink truncate" style="direction: rtl;">
				&#x200E;
				{fileLabel.name}
				{#if fileLabel.disambiguator}
					<span class="text-primary-400">
						({fileLabel.disambiguator})
					</span>
				{/if}
			</div>

			<IconButton
				label={m.configFileListItem_button_openFile()}
				icon="mdi:open-in-new"
				class="mr-1 ml-2 hidden group-hover:block"
				onclick={(evt) => {
					evt.preventDefault();
					openFile();
				}}
			/>

			{#if !locked}
				<IconButton
					label={m.configFileListItem_button_deleteFile()}
					icon="mdi:delete"
					class="hidden group-hover:block"
					onclick={(evt) => {
						evt.preventDefault();
						deleteFile();
					}}
				/>
			{/if}
		</Collapsible.Trigger>
		<Collapsible.Content class="text-primary-300 mb-1">
			{#each shownSections as section}
				<button
					class={[
						config.selectedSection === section ? 'bg-primary-700' : 'hover:bg-primary-700',
						'block w-full truncate rounded py-0.5 pr-2 pl-10 text-left'
					]}
					onclick={() => {
						config.selectedFile = file;
						config.selectedSection = section;
					}}
				>
					{section.name}
				</button>
			{/each}
		</Collapsible.Content>
	</Collapsible.Root>
{/if}
