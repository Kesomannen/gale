<script lang="ts">
	import Icon from '@iconify/svelte';
	import type { DropdownOption, Mod } from '../models';
	import { isOutdated as updateAvailable, shortenNum } from '../util';
	import { Button, DropdownMenu } from 'bits-ui';
	import { slide } from 'svelte/transition';
	import Popup from '$lib/Popup.svelte';
	import Markdown from '$lib/Markdown.svelte';

	import { open } from '@tauri-apps/api/shell';
	import { fetch, Response } from '@tauri-apps/api/http';
	import { currentGame } from '$lib/profile';
	import { get } from 'svelte/store';
	import ModInfoPopup from './ModInfoPopup.svelte';

	export let mod: Mod;
	export let onClose: () => void;
	export let extraDropdownOptions: DropdownOption[] = [];

	const defaultDropdownOptions: DropdownOption[] = [
		{
			icon: 'mdi:close',
			label: 'Close',
			onClick: onClose
		}
	];

	let dropdownOptions = defaultDropdownOptions;

	$: {
		let options = [...defaultDropdownOptions];

		if (extraDropdownOptions.length > 0) {
			options.splice(0, 0, ...extraDropdownOptions);
		}

		if (mod.websiteUrl && mod.websiteUrl.length > 0) {
			options.splice(0, 0, {
				icon: 'mdi:open-in-new',
				label: 'Open website',
				onClick: () => open(mod.websiteUrl!)
			});
		}

		if (mod.donateUrl) {
			options.splice(0, 0, {
				icon: 'mdi:heart',
				label: 'Donate',
				onClick: () => open(mod.donateUrl!)
			});
		}

		dropdownOptions = options;
	}

	let dependenciesOpen = false;

	let readmeOpen = false;
	let readme: ModInfoPopup;

	let changelogOpen = false;
	let changelog: ModInfoPopup;

	function openCommunityUrl(tail?: string) {
		if (!tail) return;

		let game = get(currentGame);
		if (!game) return;
		
		open(`https://thunderstore.io/c/${game.id}/p/${tail}/`)
	}
</script>

<div
	class="flex flex-col px-6 pt-6 pb-4 min-w-80 w-[40%] bg-gray-700 text-white border-l border-gray-600 relative"
>
	<DropdownMenu.Root>
		<DropdownMenu.Trigger class="absolute right-2 top-18 rounded-full hover:bg-slate-600 p-1">
			<Icon class="text-slate-200 text-2xl" icon="mdi:dots-vertical" />
		</DropdownMenu.Trigger>
		<DropdownMenu.Content
			class="flex flex-col bg-gray-700 gap-0.5 shadow-xl p-1 rounded-lg border border-gray-500"
			transition={slide}
			transitionConfig={{ duration: 100 }}
		>
			{#each dropdownOptions as option}
				<DropdownMenu.Item
					class="flex items-center pl-3 pr-5 py-1 truncate text-slate-300 hover:text-slate-100 text-left rounded-md hover:bg-gray-600 cursor-default"
					on:click={option.onClick}
				>
					{#if option.icon}
						<Icon class="text-xl mr-1" icon={option.icon} />
					{/if}
					{option.label}
				</DropdownMenu.Item>
			{/each}
		</DropdownMenu.Content>
	</DropdownMenu.Root>

	<div class="mr-8 flex items-center justify-between">
		<Button.Root
			class="text-slate-200 font-semibold text-2xl hover:underline truncate"
			on:click={() => openCommunityUrl(mod.author + '/' + mod.name)}>{mod.name}</Button.Root
		>
		{#if mod.version}
			<span class="text-slate-300 font-light text-lg pl-2 align-middle"
			>{mod.version}</span
		>
		{/if}
	</div>

	{#if mod.author}
		<span class="text-slate-400 text-lg">
			By
			<Button.Root
				class="hover:underline"
				on:click={() => openCommunityUrl(mod.author)}
			>
				{mod.author}
			</Button.Root>
		</span>
	{/if}

	{#if mod.isDeprecated}
		<div class="flex items-center rounded-lg bg-red-600 text-white px-3 py-1 max-w-fit my-1">
			<Icon class="text-xl mr-1" icon="mdi:error" />
			Deprecated
		</div>
	{/if}

	{#if mod.enabled === false}
		<div class="flex items-center rounded-lg bg-yellow-400 text-slate-800 px-3 py-1 max-w-fit my-1">
			<Icon class="text-xl mr-1" icon="mdi:eye-off" />
			Disabled
		</div>
	{/if}

	{#if mod.categories}
		<div class="flex gap-1 mt-2 flex-wrap mb-3">
			{#each mod.categories as category}
				<div class="bg-slate-600 rounded-full px-3 py-1 text-blue-100 text-sm">
					{category}
				</div>
			{/each}
		</div>
	{/if}

	{#if mod.rating && mod.downloads}
		<div>
			<div class="inline-flex items-center gap-2 mr-6">
				<Icon class="text-yellow-400 text-lg" icon="mdi:star" />
				<span class="text-yellow-400 text-md">{shortenNum(mod.rating)}</span>
			</div>
			<div class="inline-flex items-center gap-2">
				<Icon class="text-green-400 text-lg" icon="mdi:download" />
				<span class="text-green-400 text-md">{shortenNum(mod.downloads)}</span>
			</div>
		</div>
	{/if}

	<p class="text-slate-300 text-lg flex-shrink overflow-hidden">{mod.description ?? ""}</p>

	<Button.Root
		class="flex items-center w-full mt-auto text-white pl-3 pr-1.5 py-1 rounded-md bg-slate-600 hover:bg-slate-500 group"
		on:mouseenter={changelog.fetchMarkdown}
		on:click={() => (changelogOpen = true)}
	>
		<Icon icon="mdi:file-document" class="text-lg mr-1" />
		Changelog
	</Button.Root>

	<Button.Root
		class="flex items-center w-full mt-1 text-white pl-3 pr-1.5 py-1 rounded-md bg-slate-600 hover:bg-slate-500 group"
		on:mouseenter={readme.fetchMarkdown}
		on:click={() => (readmeOpen = true)}
	>
		<Icon icon="mdi:info" class="text-lg mr-1" />
		Details
	</Button.Root>

	{#if mod.dependencies && mod.dependencies.length > 0}
		<Button.Root
			class="flex items-center w-full mt-1 text-white pl-3 pr-1 py-1 rounded-md bg-slate-600 hover:bg-slate-500 group"
			on:click={() => (dependenciesOpen = true)}
		>
			<Icon icon="material-symbols:network-node" class="text-lg mr-1" />
			Dependencies
			<div class="bg-slate-500 group-hover:bg-slate-400 px-3 py-0.5 text-sm rounded-md ml-auto">
				{mod.dependencies.length}
			</div>
		</Button.Root>
	{/if}

	<slot />
</div>

<Popup title="Dependencies of {mod.name}" bind:open={dependenciesOpen}>
	{#if mod.dependencies}
		<table class="mt-2 w-full">
			<tr class="text-slate-100 text-left">
				<th>Author</th>
				<th>Name</th>
				<th>Preferred Version</th>
			</tr>
			{#each mod.dependencies as dependency}
				<tr class="text-slate-200 even:bg-gray-700">
					{#each dependency.split('-') as segment}
						<td class="pr-4 pl-2">{segment}</td>
					{/each}
				</tr>
			{/each}
		</table>
	{/if}
</Popup>

<ModInfoPopup bind:this={readme} bind:open={readmeOpen} {mod} path="readme" />
<ModInfoPopup bind:this={changelog} bind:open={changelogOpen} {mod} path="changelog" />
