<script lang="ts">
	import * as api from '$lib/api';
	import IconButton from '$lib/components/ui/IconButton.svelte';
	import { m } from '$lib/paraglide/messages';
	import type { ConfigFile } from '$lib/types';
	import Icon from '@iconify/svelte';
	import { confirm } from '@tauri-apps/plugin-dialog';

	type Props = {
		file: ConfigFile;
		duplicate: boolean;
		locked: boolean;
		selected: boolean;
		onDeleteClicked: () => void;
		onFileClicked: (file: ConfigFile) => void;
	};

	let { file, duplicate, locked, selected, onDeleteClicked, onFileClicked }: Props = $props();

	let { colorClasses, icon } = $derived(
		{
			ok: { colorClasses: 'text-primary-200', icon: null },
			err: { colorClasses: 'text-red-400', icon: 'ph:warning-circle-fill' },
			unsupported: { colorClasses: 'text-primary-400', icon: 'ph:question-fill' }
		}[file.type]
	);

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

		await api.config.deleteFile(file);
		onDeleteClicked();
	}

	async function openFile() {
		await api.config.openFile(file);
	}
</script>

{#if file.type !== 'ok' || shownSections.length > 0}
	<button
		onclick={() => onFileClicked(file)}
		class={[
			colorClasses,
			selected ? 'bg-primary-700 font-semibold' : 'hover:bg-primary-700',
			'group flex w-full items-center overflow-hidden rounded px-2 py-1'
		]}
	>
		{#if icon}
			<Icon {icon} class="mr-1 shrink-0 text-lg" />
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
			icon="ph:arrow-square-out-fill"
			class="mr-1 ml-2 hidden group-hover:block"
			onclick={(evt) => {
				evt.preventDefault();
				openFile();
			}}
		/>

		{#if !locked}
			<IconButton
				label={m.configFileListItem_button_deleteFile()}
				icon="ph:trash-fill"
				class="hidden group-hover:block"
				onclick={(evt) => {
					evt.preventDefault();
					deleteFile();
				}}
			/>
		{/if}
	</button>
{/if}
