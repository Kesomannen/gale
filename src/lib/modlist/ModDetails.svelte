<script lang="ts">
	import Popup from '$lib/components/Popup.svelte';
	import Markdown from '$lib/components/Markdown.svelte';

	import ModInfoPopup from './ModInfoPopup.svelte';
	import ModCardList from './ModCardList.svelte';
	import ModContextMenuItems from './ModContextMenuItems.svelte';

	import { activeGame } from '$lib/stores';
	import type { MarkdownResponse, Mod, ModContextItem } from '$lib/models';
	import { communityUrl, shortenFileSize, shortenNum, timeSince } from '$lib/util';

	import { Button, DropdownMenu } from 'bits-ui';

	import { get } from 'svelte/store';

	import { fetch } from '@tauri-apps/plugin-http';
	import { open } from '@tauri-apps/plugin-shell';

	import Icon from '@iconify/svelte';
	import { dropTransition } from '$lib/transitions';
	import { createEventDispatcher } from 'svelte';

	export let mod: Mod;
	export let contextItems: ModContextItem[] = [];

	const dispatch = createEventDispatcher<{ close: void }>();

	let dependenciesOpen = false;

	let readmeOpen = false;
	let readme: ModInfoPopup;

	let changelogOpen = false;
	let changelog: ModInfoPopup;

	$: allContextItems = [
		...contextItems,
		{
			label: 'Close',
			icon: 'mdi:close',
			onclick: () => dispatch('close')
		}
	];

	let readmePromise: Promise<string | null>;

	async function extractReadme(response: Response) {
		let res = (await response.json()) as MarkdownResponse;

		if (!res.markdown) return null;

		console.log('extracted readme');

		return res.markdown
			.split('\n')
			.filter((line) => !line.startsWith('# '))
			.join('\n');
	}

	$: {
		let url = `https://thunderstore.io/api/experimental/package/${mod.author}/${mod.name}/${mod.version}/readme/`;
		readmePromise = fetch(url).then(extractReadme);
	}
</script>

<div
	class="relative flex w-[40%] min-w-72 flex-col border-l border-slate-600 bg-slate-700 px-6 pb-4 pt-6 text-white"
>
	<DropdownMenu.Root>
		<DropdownMenu.Trigger class="absolute right-2 mt-0.5 rounded-full p-1 hover:bg-slate-600">
			<Icon class="text-2xl text-slate-200" icon="mdi:dots-vertical" />
		</DropdownMenu.Trigger>
		<DropdownMenu.Content
			class="flex flex-col gap-0.5 rounded-lg border border-slate-500 bg-slate-700 p-1 shadow-xl"
			{...dropTransition}
		>
			<ModContextMenuItems {mod} contextItems={allContextItems} type="details" />
		</DropdownMenu.Content>
	</DropdownMenu.Root>

	<a
		class="break-words pr-4 text-left text-3xl font-bold text-white hover:underline xl:text-4xl"
		href={communityUrl(mod.author + '/' + mod.name)}
		target="_blank">{mod.name.replace(/_/g, ' ')}</a
	>

	{#if mod.author}
		<div class="text-xl text-slate-300 xl:text-2xl">
			By
			<a class="hover:underline" href={communityUrl(mod.author)} target="_blank">
				{mod.author}
			</a>
		</div>
	{/if}

	{#if mod.version}
		<div class="text-xl text-slate-300 xl:text-2xl">v{mod.version}</div>
	{/if}

	<div class="flex flex-wrap gap-1">
		{#if mod.isDeprecated}
			<div class="my-1 flex items-center rounded-lg bg-red-600 px-3 py-1 text-white">
				<Icon class="mr-1 text-xl" icon="mdi:error" />
				Deprecated
			</div>
		{/if}

		{#if mod.containsNsfw}
			<div class="my-1 flex items-center rounded-lg bg-red-600 px-3 py-1 text-white">
				<Icon class="mr-1 text-xl" icon="material-symbols:explicit" />
				Contains NSFW
			</div>
		{/if}
	</div>

	{#if mod.categories}
		<div class="text-md my-2 flex flex-wrap gap-1">
			{#each mod.categories as category}
				<div class="rounded-full bg-slate-600 px-4 py-1 text-slate-200">
					{category}
				</div>
			{/each}
		</div>
	{/if}

	<div class="my-1 flex items-center gap-1.5 text-lg">
		{#if mod.rating !== null}
			<Icon class="flex-shrink-0 text-yellow-400" icon="mdi:star" />
			<span class="mr-4 text-yellow-400">{shortenNum(mod.rating)}</span>
		{/if}
		{#if mod.downloads !== null}
			<Icon class="flex-shrink-0 text-green-400" icon="mdi:download" />
			<span class="mr-4 text-green-400">{shortenNum(mod.downloads)}</span>
		{/if}
		<Icon class="flex-shrink-0 text-slate-400" icon="mdi:weight" />
		<span class="text-slate-400">{shortenFileSize(mod.fileSize)}</span>
	</div>

	{#if mod.lastUpdated !== null}
		<div class="text-lg text-slate-400">
			Last updated {timeSince(new Date(mod.lastUpdated))} ago
		</div>
	{/if}

	{#if mod.description !== null}
		<p class="mt-2 flex-shrink overflow-hidden text-xl text-slate-300 lg:hidden">
			{mod.description}
		</p>
	{/if}

	{#await readmePromise}
		<div class="hidden h-full w-full items-center justify-center lg:flex">
			<Icon class="animate-spin text-5xl text-slate-300" icon="mdi:loading" />
		</div>
	{:then readme}
		{#if readme}
			<Markdown source={readme} class="readme hidden lg:block" />
		{:else}
			<p class="mt-3 hidden flex-shrink overflow-hidden text-xl text-slate-300 lg:block">
				{mod.description ?? ''}
			</p>
		{/if}
	{/await}

	<div class="flex-grow" />

	{#if mod.configFile}
		<div
			class="my-2 flex items-center gap-2 text-lg text-accent-400 hover:text-accent-300 hover:underline"
		>
			<Icon class="text-xl" icon="mdi:file-cog" />
			<a href={'/config?file=' + mod.configFile}>Edit config</a>
		</div>
	{/if}

	{#if mod.type === 'remote'}
		<Button.Root
			class="group flex items-center rounded-md bg-slate-600 py-1 pl-3 pr-1.5 text-white hover:bg-slate-500"
			on:mouseenter={changelog.fetchMarkdown}
			on:click={() => (changelogOpen = true)}
		>
			<Icon icon="mdi:file-document" class="mr-2 text-lg" />
			Changelog
		</Button.Root>

		<Button.Root
			class="group mt-1 flex items-center rounded-md bg-slate-600 py-1 pl-3 pr-1.5 text-white hover:bg-slate-500"
			on:mouseenter={readme.fetchMarkdown}
			on:click={() => (readmeOpen = true)}
		>
			<Icon icon="mdi:info" class="mr-2 text-lg" />
			Details
		</Button.Root>
	{/if}

	{#if mod.dependencies && mod.dependencies.length > 0}
		<Button.Root
			class="group mt-1 flex items-center rounded-md bg-slate-600 py-1 pl-3 pr-1 text-white hover:bg-slate-500"
			on:click={() => (dependenciesOpen = true)}
		>
			<Icon icon="material-symbols:network-node" class="mr-2 text-lg" />
			Dependencies
			<div class="ml-auto rounded-md bg-slate-500 px-3 py-0.5 text-sm group-hover:bg-slate-400">
				{mod.dependencies.length}
			</div>
		</Button.Root>
	{/if}

	<slot />
</div>

<Popup
	large={(mod.dependencies?.length ?? 0) > 10}
	title="Dependencies of {mod.name}"
	bind:open={dependenciesOpen}
>
	{#if mod.dependencies}
		<ModCardList names={mod.dependencies} class="mt-4" />
	{/if}
</Popup>

<ModInfoPopup bind:this={readme} bind:open={readmeOpen} {mod} path="readme" />
<ModInfoPopup
	bind:this={changelog}
	bind:open={changelogOpen}
	{mod}
	useLatest={true}
	path="changelog"
/>

<style lang="postcss">
	:global(.readme) {
		scrollbar-color: theme(colors.slate.400) theme(colors.slate.700);
	}
</style>
