<script lang="ts">
	import Icon from '@iconify/svelte';
	import type { DropdownOption, Mod } from '../models';
	import { getTotalDownloads, shortenNum } from '../util';
	import { Button, DropdownMenu } from 'bits-ui';
	import { slide } from 'svelte/transition';
	import Popup from '$lib/Popup.svelte';
	import Markdown from '$lib/Markdown.svelte';

	import { open } from '@tauri-apps/api/shell';
	import { fetch, Response } from '@tauri-apps/api/http';

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

		if (mod.version.websiteUrl.length > 0) {
			options.splice(0, 0, {
				icon: 'mdi:open-in-new',
				label: 'Open website',
				onClick: () => open(mod.version.websiteUrl)
			});
		}

		if (mod.package.donationLink) {
			options.splice(0, 0, {
				icon: 'mdi:heart',
				label: 'Donate',
				onClick: () => open(mod.package.donationLink!)
			});
		}

		dropdownOptions = options;
	}

	let dependenciesOpen = false;
	let readmeOpen = false;
	let readmePromise: Promise<Response<MarkdownResponse>> | null = null;
	let currentReadme: Mod | null = null;

	interface MarkdownResponse {
		markdown: string;
	}

	function fetchReadme() {
		if (currentReadme === mod) return;

		let url = `https://thunderstore.io/api/experimental/package/${mod.package.owner}/${mod.package.name}/${mod.version.versionNumber}/readme/`;
		readmePromise = fetch<MarkdownResponse>(url, { method: 'GET' })
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
			on:click={() =>
				open(
					`https://thunderstore.io/c/lethal-company/p/${mod.package.owner}/${mod.package.name}/`
				)}>{mod.version.name}</Button.Root
		>
		<span class="text-slate-300 font-light text-lg align-middle ml-1"
			>v{mod.version.versionNumber}</span
		>
	</div>

	<span class="text-slate-400 text-lg">
		By
		<Button.Root
			class="hover:underline"
			on:click={() => open('https://thunderstore.io/c/lethal-company/p/' + mod.package.owner)}
		>
			{mod.package.owner}
		</Button.Root>
	</span>

	<div class="flex gap-1 mt-2 flex-wrap">
		{#each mod.package.categories as category}
			<div class="bg-slate-600 rounded-full px-3 py-1 text-blue-100 text-sm">
				{category}
			</div>
		{/each}
	</div>

	<div class="mt-3">
		<div class="inline-flex items-center gap-2 mr-6">
			<Icon class="text-yellow-400 text-lg" icon="mdi:star" />
			<span class="text-yellow-400 text-md">{shortenNum(mod.package.ratingScore)}</span>
		</div>
		<div class="inline-flex items-center gap-2">
			<Icon class="text-green-400 text-lg" icon="mdi:download" />
			<span class="text-green-400 text-md">{shortenNum(getTotalDownloads(mod.package))}</span>
		</div>
	</div>

	<p class="text-slate-300 text-lg">{mod.version.description}</p>

	<Button.Root
		class="flex items-center w-full mt-auto text-white pl-3 pr-1.5 py-1 rounded-md bg-gray-600 hover:bg-gray-500 group"
		on:mouseenter={fetchReadme}
		on:click={() => (readmeOpen = true)}
	>
		<Icon icon="mdi:info" class="text-lg mr-1" />
		Details
	</Button.Root>

	{#if mod.version.dependencies.length > 0}
		<Button.Root
			class="flex items-center w-full mt-1 text-white pl-3 pr-1 py-1 rounded-md bg-gray-600 hover:bg-gray-500 group"
			on:click={() => (dependenciesOpen = true)}
		>
			<Icon icon="material-symbols:network-node" class="text-lg mr-1" />
			Dependencies
			<div class="bg-gray-500 group-hover:bg-gray-400 px-3 py-0.5 text-sm rounded-md ml-auto">
				{mod.version.dependencies.length}
			</div>
		</Button.Root>
	{/if}

	<slot />
</div>

<Popup title="Dependencies of {mod.package.name}" bind:open={dependenciesOpen}>
	<table class="mt-2">
		<tr class="text-slate-100 text-left">
			<th>Author</th>
			<th>Name</th>
			<th>Preferred Version</th>
		</tr>
		{#each mod.version.dependencies as dependency}
			<tr class="text-slate-300">
				{#each dependency.split('-') as segment}
					<td class="pr-4">{segment}</td>
				{/each}
			</tr>
		{/each}
	</table>
</Popup>

<Popup bind:open={readmeOpen}>
	{#await readmePromise}
		<Icon class="text-slate-300 text-4xl animate-spin" icon="mdi:loading" />
	{:then value}
		{#if value?.ok}
			<Markdown source={value.data.markdown} />
		{:else}
			<p class="text-red-300">Failed to load README: error code {value?.status}</p>
		{/if}
	{:catch error}
		<p class="text-red-300">Failed to load README: {error}</p>
	{/await}
</Popup>
